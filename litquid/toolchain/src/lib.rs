//! LitQuid preprocessor library - converts Liquid templates to Lit template modules

use std::fs;
use std::path::Path;

pub mod codegen;

/// Default Lit import path (CDN)
pub const DEFAULT_LIT_IMPORT: &str = "lit";

/// A dynamic binding extracted from a `csr` filter expression.
#[derive(Debug, Clone)]
pub struct Binding {
    /// Root variable name (first pipe segment of the Liquid expression)
    pub var_name: String,
    /// Ordered filter chain applied before the `csr` marker
    pub filters: Vec<(String, Vec<String>)>,
}

/// All information extracted from a single `.liquid` template file.
///
/// Carries everything needed to emit a JS module, a JSON manifest, and
/// server-side render functions in any target language.
pub struct ParsedTemplate {
    /// Base64 digest matching `@lit-labs/ssr-client`'s `digestForTemplateResult`
    pub digest: String,
    /// Static HTML segments between dynamic bindings (length = `bindings.len() + 1`).
    /// Computed after stripping the outer `<template>` wrapper.
    pub static_segments: Vec<String>,
    /// Dynamic bindings in document order, one per `${...}` position in the template.
    pub bindings: Vec<Binding>,
    /// Opening `<template ...>` tag from the source file, if present.
    pub template_open_tag: Option<String>,
    // Stored for JS module regeneration
    processed_html: String,
    lit_import: String,
}

impl ParsedTemplate {
    /// Generate the `.template.js` module content.
    pub fn to_js_module(&self) -> String {
        generate_js_module_with_digest(&self.processed_html, &self.lit_import, &self.digest)
    }

    /// Generate the `.template.json` manifest content.
    pub fn to_json_manifest(&self) -> String {
        format!("{{\"digest\":\"{}\"}}\n", self.digest)
    }
}

/// Process a `.liquid` file and return the fully parsed template.
pub fn process_liquid_file(path: &Path, lit_import: Option<&str>) -> Result<ParsedTemplate, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lit_import_str = lit_import.unwrap_or(DEFAULT_LIT_IMPORT);

    let (processed_html, bindings) = process_liquid_content(&content)?;

    let template_open_tag = extract_template_open_tag(&processed_html);
    let client_content = strip_template_wrapper(&processed_html);
    let static_segments = split_template_strings(client_content.trim());
    let digest = compute_template_digest(&static_segments);

    Ok(ParsedTemplate {
        digest,
        static_segments,
        bindings,
        template_open_tag,
        processed_html,
        lit_import: lit_import_str.to_string(),
    })
}

/// Process liquid content using character-based parsing.
/// Returns `(processed_html, bindings)` where processed_html has `${...}` expressions
/// in place of `{{ ... | csr }}` blocks, and bindings carries the structured metadata
/// for each dynamic position.
pub fn process_liquid_content(content: &str) -> Result<(String, Vec<Binding>), String> {
    let mut result = String::new();
    let mut bindings: Vec<Binding> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < len && !(chars[j] == '}' && chars[j + 1] == '}') {
                j += 1;
            }
            if j + 1 < len {
                let expr: String = chars[start..j].iter().collect();
                let (extracted, mut block_bindings) = process_liquid_block(&expr);
                result.push_str(&extracted);
                bindings.append(&mut block_bindings);
                i = j + 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    Ok((result, bindings))
}

/// Split processed HTML (with ${...} expressions) into static template strings.
///
/// This matches the `strings` array that the JS `html` tagged template literal receives —
/// the static string parts between each `${...}` interpolation.
pub fn split_template_strings(html: &str) -> Vec<String> {
    let mut strings: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '$' && i + 1 < len && chars[i + 1] == '{' {
            strings.push(current.clone());
            current = String::new();
            i += 2; // skip '${'
            let mut depth = 1;
            while i < len && depth > 0 {
                match chars[i] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }
    strings.push(current);
    strings
}

/// Compute the Lit template digest matching `@lit-labs/ssr-client`'s `digestForTemplateResult`.
///
/// Algorithm: DJB2-inspired with two 32-bit accumulators (init 5381) and UTF-16 code units.
/// The index `i` resets at the start of each string, matching the JS loop exactly.
/// Output is base64-encoded little-endian bytes of the two u32 accumulators (8 bytes → 12 chars).
pub fn compute_template_digest(strings: &[String]) -> String {
    let mut hashes = [5381u32, 5381u32];

    for s in strings {
        for (i, unit) in s.encode_utf16().enumerate() {
            let idx = i % 2;
            hashes[idx] = ((hashes[idx] as u64 * 33) as u32) ^ (unit as u32);
        }
    }

    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&hashes[0].to_le_bytes());
    bytes[4..8].copy_from_slice(&hashes[1].to_le_bytes());

    base64_encode(&bytes)
}

/// Generate an SSR wrapper for the processed template HTML.
pub fn generate_ssr_wrapper(processed_html: &str, digest: &str) -> String {
    let client_content = strip_template_wrapper(processed_html);
    let with_slots = replace_expressions_with_slots(client_content.trim());
    format!("<!--lit-part {}-->\n{}\n<!--/lit-part-->", digest, with_slots)
}

/// Generate the JS module content, embedding the pre-computed digest.
fn generate_js_module_with_digest(html_content: &str, lit_import: &str, digest: &str) -> String {
    let client_content = strip_template_wrapper(html_content);
    let trimmed = client_content.trim();

    format!(
        "import {{ html }} from '{lit_import}';\n\nexport const templateDigest = '{digest}';\n\n/**\n * Returns the template for this component.\n * Use with LitQuidElement.renderTemplate() for CSR rendering.\n */\nexport function getTemplate(ctx) {{\n  return html`{content}`;\n}}\n",
        lit_import = lit_import,
        digest = digest,
        content = trimmed,
    )
}

/// Generate the JS module content (digest computed internally).
pub fn generate_js_module(html_content: &str, lit_import: &str) -> String {
    let client_content = strip_template_wrapper(html_content);
    let strings = split_template_strings(client_content.trim());
    let digest = compute_template_digest(&strings);
    generate_js_module_with_digest(html_content, lit_import, &digest)
}

/// Base64-encode bytes (RFC 4648, standard alphabet with `=` padding).
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        out.push(ALPHA[((b0 >> 2) & 0x3f) as usize] as char);
        out.push(ALPHA[(((b0 << 4) | (b1 >> 4)) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHA[(((b1 << 2) | (b2 >> 6)) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHA[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Replace `${...}` expressions with `<!--lit-part--><!--/lit-part-->` slots.
fn replace_expressions_with_slots(html: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '$' && i + 1 < len && chars[i + 1] == '{' {
            result.push_str("<!--lit-part--><!--/lit-part-->");
            i += 2;
            let mut depth = 1;
            while i < len && depth > 0 {
                match chars[i] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Process one `{{ ... }}` block, returning the JS expression(s) and any binding metadata.
fn process_liquid_block(expr: &str) -> (String, Vec<Binding>) {
    if !expr.contains("csr") {
        return (String::new(), vec![]);
    }

    let segments = split_pipe(expr);
    let mut js_result = String::new();
    let mut bindings = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        let trimmed = segment.trim();
        if !trimmed.starts_with("csr") {
            continue;
        }
        if i == 0 {
            continue; // csr as first segment has no variable
        }

        let var_name = segments[0].trim().to_string();
        let filters: Vec<(String, Vec<String>)> = segments[1..i]
            .iter()
            .map(|s| parse_filter(s.trim()))
            .collect();

        let rest = trimmed["csr".len()..].trim();
        let js_expr = if rest.starts_with(':') {
            // Explicit arg — extract quoted string and normalise `this.` → `ctx.`
            let after_colon = rest[1..].trim();
            if after_colon.starts_with('"') {
                let inner = &after_colon[1..];
                let inner_chars: Vec<char> = inner.chars().collect();
                let mut j = 0;
                while j < inner_chars.len() && inner_chars[j] != '"' {
                    if inner_chars[j] == '\\' && j + 1 < inner_chars.len() {
                        j += 2;
                    } else {
                        j += 1;
                    }
                }
                let arg: String = inner_chars[..j].iter().collect();
                arg.replace("this.", "ctx.")
            } else {
                String::new()
            }
        } else {
            // No arg — auto-translate the filter chain to JS
            let translated = translate_filter_chain(&var_name, &filters);
            format!("${{{}}}", translated)
        };

        js_result.push_str(&js_expr);
        bindings.push(Binding { var_name, filters });
    }

    (js_result, bindings)
}

/// Extract the opening `<template ...>` tag from content, if present.
fn extract_template_open_tag(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.starts_with("<template") {
        if let Some(end) = trimmed.find('>') {
            return Some(trimmed[..=end].to_string());
        }
    }
    None
}

/// Split an expression on `|` separators, respecting quoted strings
fn split_pipe(expr: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '"' => {
                in_quotes = !in_quotes;
                current.push('"');
                i += 1;
            }
            '\\' if in_quotes => {
                if i + 1 < chars.len() {
                    current.push('\\');
                    current.push(chars[i + 1]);
                    i += 2;
                } else {
                    current.push('\\');
                    i += 1;
                }
            }
            '|' if !in_quotes => {
                segments.push(current.trim().to_string());
                current = String::new();
                i += 1;
            }
            c => {
                current.push(c);
                i += 1;
            }
        }
    }
    let tail = current.trim().to_string();
    if !tail.is_empty() {
        segments.push(tail);
    }
    segments
}

/// Parse a filter segment like `"plus: 1"` into `("plus", ["1"])`
fn parse_filter(segment: &str) -> (String, Vec<String>) {
    match segment.find(':') {
        Some(pos) => {
            let name = segment[..pos].trim().to_string();
            let arg = segment[pos + 1..].trim().to_string();
            (name, vec![arg])
        }
        None => (segment.trim().to_string(), vec![]),
    }
}

/// Build a JS expression by starting from `ctx.<var>` and applying each filter in order
fn translate_filter_chain(var: &str, filters: &[(String, Vec<String>)]) -> String {
    let mut expr = format!("ctx.{}", var);
    for (name, args) in filters {
        expr = apply_filter(&expr, name, args);
    }
    expr
}

/// Translate a single Liquid filter application to its JS equivalent.
/// Unknown filters emit a warning to stderr and are dropped.
fn apply_filter(expr: &str, name: &str, args: &[String]) -> String {
    match name {
        "capitalize" => format!(
            "({e}.charAt(0).toUpperCase()+{e}.slice(1).toLowerCase())",
            e = expr
        ),
        "upcase" => format!("{}.toUpperCase()", expr),
        "downcase" => format!("{}.toLowerCase()", expr),
        "strip" => format!("{}.trim()", expr),
        "plus" => format!("({}+{})", expr, args[0]),
        "minus" => format!("({}-{})", expr, args[0]),
        "times" => format!("({}*{})", expr, args[0]),
        "divided_by" => format!("({}/{})", expr, args[0]),
        "modulo" => format!("({}%{})", expr, args[0]),
        "floor" => format!("Math.floor({})", expr),
        "ceil" => format!("Math.ceil({})", expr),
        "round" => format!("Math.round({})", expr),
        "abs" => format!("Math.abs({})", expr),
        "append" => format!("({}+{})", expr, args[0]),
        "prepend" => format!("({}+{})", args[0], expr),
        "default" => format!("({}??{})", expr, args[0]),
        "size" => format!("{}.length", expr),
        "at_least" => format!("Math.max({},{})", expr, args[0]),
        "at_most" => format!("Math.min({},{})", expr, args[0]),
        "truncate" => format!("({e}.length>{n}?{e}.slice(0,{n})+'...':String({e}))", e = expr, n = args.get(0).map(|s| s.as_str()).unwrap_or("50")),
        "replace" => format!("{}.replaceAll({},{})", expr, args.get(0).map(|s| s.as_str()).unwrap_or("\"\""), args.get(1).map(|s| s.as_str()).unwrap_or("\"\"")),
        "remove" => format!("{}.replaceAll({},\"\")", expr, args.get(0).map(|s| s.as_str()).unwrap_or("\"\"")),
        "join" => format!("{}.join({})", expr, args.get(0).map(|s| s.as_str()).unwrap_or("\", \"")),
        "first" => format!("{}[0]", expr),
        "last" => format!("{}[{}.length-1]", expr, expr),
        "reverse" => format!("[...{}].reverse()", expr),
        "url_encode" => format!("encodeURIComponent({})", expr),
        "url_decode" => format!("decodeURIComponent({})", expr),
        unknown => {
            eprintln!(
                "litquid warning: unknown filter '{}', dropping — add explicit csr arg to suppress",
                unknown
            );
            expr.to_string()
        }
    }
}

/// Strip the <template shadowrootmode="..."> wrapper from content
fn strip_template_wrapper(content: &str) -> String {
    let trimmed = content.trim();

    if trimmed.starts_with("<template") && trimmed.ends_with("</template>") {
        if let Some(open_end) = trimmed.find('>') {
            if let Some(close_start) = trimmed.rfind("</template>") {
                return trimmed[open_end + 1..close_start].to_string();
            }
        }
    }

    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- csr with explicit arg ---

    #[test]
    fn test_csr_explicit_arg() {
        let input = r#"<div>
  {{ firstName | capitalize | csr: "${this.firstName}" }}
  {{ lastName | csr: "${this.lastName}" }}
  {{ age | plus: 1 }}
</div>"#;

        let expected = "<div>\n  ${ctx.firstName}\n  ${ctx.lastName}\n  \n</div>";

        let (result, bindings) = process_liquid_content(input).unwrap();
        assert_eq!(result, expected);
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].var_name, "firstName");
        assert_eq!(bindings[1].var_name, "lastName");
    }

    #[test]
    fn test_csr_explicit_arg_multiple_in_one_expression() {
        let input = r#"{{ a | csr: "${this.a}" | csr: "${this.b}" }}"#;
        let (result, bindings) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.a}${ctx.b}");
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_csr_explicit_arg_arbitrary_interpolation() {
        let input = r#"{{ items | csr: "${this.items.map(i => i.name).join(', ')}" }}"#;
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.items.map(i => i.name).join(', ')}");
    }

    #[test]
    fn test_csr_explicit_arg_complex_expression() {
        let input = r#"{{ x | csr: "${this.fn(a, b)}" }}"#;
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.fn(a, b)}");
    }

    // --- csr without arg (auto-translation) ---

    #[test]
    fn test_csr_no_arg_no_filters() {
        let input = "{{ name | csr }}";
        let (result, bindings) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.name}");
        assert_eq!(bindings[0].var_name, "name");
        assert!(bindings[0].filters.is_empty());
    }

    #[test]
    fn test_csr_no_arg_capitalize() {
        let input = "{{ firstName | capitalize | csr }}";
        let (result, bindings) = process_liquid_content(input).unwrap();
        assert_eq!(
            result,
            "${(ctx.firstName.charAt(0).toUpperCase()+ctx.firstName.slice(1).toLowerCase())}"
        );
        assert_eq!(bindings[0].filters[0].0, "capitalize");
    }

    #[test]
    fn test_csr_no_arg_upcase() {
        let input = "{{ code | upcase | csr }}";
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.code.toUpperCase()}");
    }

    #[test]
    fn test_csr_no_arg_plus() {
        let input = "{{ age | plus: 1 | csr }}";
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${(ctx.age+1)}");
    }

    #[test]
    fn test_csr_no_arg_chained_times_round() {
        let input = "{{ price | times: 1.2 | round | csr }}";
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${Math.round((ctx.price*1.2))}");
    }

    #[test]
    fn test_csr_no_arg_default() {
        let input = r#"{{ title | default: "Untitled" | csr }}"#;
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, r#"${(ctx.title??"Untitled")}"#);
    }

    #[test]
    fn test_csr_no_arg_size() {
        let input = "{{ items | size | csr }}";
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.items.length}");
    }

    #[test]
    fn test_csr_no_arg_unknown_filter_drops_it() {
        let input = "{{ name | some_custom_filter | csr }}";
        let (result, _) = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.name}");
    }

    // --- no csr filter at all ---

    #[test]
    fn test_no_csr_filter() {
        let input = "{{ age | plus: 1 }}";
        let (result, bindings) = process_liquid_content(input).unwrap();
        assert_eq!(result, "");
        assert!(bindings.is_empty());
    }

    // --- split_template_strings ---

    #[test]
    fn test_split_no_expressions() {
        let strings = split_template_strings("<div>hello</div>");
        assert_eq!(strings, vec!["<div>hello</div>"]);
    }

    #[test]
    fn test_split_one_expression() {
        let strings = split_template_strings("<div>${ctx.name}</div>");
        assert_eq!(strings, vec!["<div>", "</div>"]);
    }

    #[test]
    fn test_split_multiple_expressions() {
        let strings = split_template_strings("<h2>${ctx.first} ${ctx.last}</h2>");
        assert_eq!(strings, vec!["<h2>", " ", "</h2>"]);
    }

    #[test]
    fn test_split_nested_braces() {
        let strings = split_template_strings("A${ctx.fn({x:1})}B");
        assert_eq!(strings, vec!["A", "B"]);
    }

    // --- compute_template_digest ---

    #[test]
    fn test_digest_is_12_chars() {
        let strings = vec!["<div>".to_string(), "</div>".to_string()];
        let digest = compute_template_digest(&strings);
        assert_eq!(digest.len(), 12);
    }

    #[test]
    fn test_digest_same_strings_same_digest() {
        let s1 = vec!["hello ".to_string(), " world".to_string()];
        let s2 = vec!["hello ".to_string(), " world".to_string()];
        assert_eq!(compute_template_digest(&s1), compute_template_digest(&s2));
    }

    #[test]
    fn test_digest_different_strings_different_digest() {
        let s1 = vec!["<div>".to_string(), "</div>".to_string()];
        let s2 = vec!["<span>".to_string(), "</span>".to_string()];
        assert_ne!(compute_template_digest(&s1), compute_template_digest(&s2));
    }

    #[test]
    fn test_digest_empty_strings() {
        let strings = vec!["".to_string()];
        let digest = compute_template_digest(&strings);
        assert_eq!(digest.len(), 12);
    }

    // --- generate_ssr_wrapper ---

    #[test]
    fn test_ssr_wrapper_no_expressions() {
        let html = "<div>hello</div>";
        let digest = "TESTDIGEST==";
        let wrapper = generate_ssr_wrapper(html, digest);
        assert!(wrapper.starts_with("<!--lit-part TESTDIGEST==-->"));
        assert!(wrapper.contains("<div>hello</div>"));
        assert!(wrapper.ends_with("<!--/lit-part-->"));
    }

    #[test]
    fn test_ssr_wrapper_replaces_expressions() {
        let html = "<p>${ctx.name}</p>";
        let wrapper = generate_ssr_wrapper(html, "HASH");
        assert!(wrapper.contains("<!--lit-part--><!--/lit-part-->"));
        assert!(!wrapper.contains("${ctx.name}"));
    }

    // --- generate_js_module ---

    #[test]
    fn test_js_module_includes_digest_export() {
        let html = "<div>${ctx.name}</div>";
        let js = generate_js_module(html, "lit");
        assert!(js.contains("export const templateDigest ="));
    }

    #[test]
    fn test_js_module_strips_template_wrapper() {
        let html = "<template shadowrootmode=\"open\"><div>${ctx.x}</div></template>";
        let js = generate_js_module(html, "lit");
        assert!(!js.contains("<template"));
        assert!(js.contains("<div>"));
    }
}
