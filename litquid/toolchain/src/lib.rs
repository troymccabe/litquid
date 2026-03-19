//! LitQuid preprocessor library - converts Liquid templates to Lit template modules

use std::fs;
use std::path::Path;

/// Default Lit import path (CDN)
pub const DEFAULT_LIT_IMPORT: &str = "lit";

/// Process a .liquid file and generate a Lit template JS module
pub fn process_liquid_file(path: &Path, lit_import: Option<&str>) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let processed_html = process_liquid_content(&content)?;

    Ok(generate_js_module(&processed_html, lit_import.unwrap_or(DEFAULT_LIT_IMPORT)))
}

/// Process liquid content using character-based parsing
/// We use a two-pass approach:
/// 1. First pass: identify {{ }} blocks and extract clientTemplateValue args
/// 2. Replace the entire {{ }} block with just the extracted arg (or empty if none)
pub fn process_liquid_content(content: &str) -> Result<String, String> {
    let mut result = String::new();
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for {{
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            // Find the matching }}
            let start = i + 2;
            let mut j = start;
            while j + 1 < len && !(chars[j] == '}' && chars[j + 1] == '}') {
                j += 1;
            }
            if j + 1 < len {
                // Extract expression content
                let expr: String = chars[start..j].iter().collect();
                let extracted = extract_client_template_value_arg(&expr);
                result.push_str(&extracted);
                i = j + 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    Ok(result)
}

/// Extract clientTemplateValue filter argument from an expression string
/// Looks for: clientTemplateValue: "..." pattern (Liquid syntax) and extracts the argument
/// Replaces `this.` with `ctx.` for use in the generated template function
fn extract_client_template_value_arg(expr: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = expr.chars().collect();
    let len = chars.len();
    let target = "clientTemplateValue";
    let target_chars: Vec<char> = target.chars().collect();
    let mut i = 0;

    while i < len {
        // Try to match "clientTemplateValue"
        if matches_at(&chars, i, &target_chars) {
            i += target_chars.len();
            // Skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            // Expect : (Liquid filter argument syntax)
            if i < len && chars[i] == ':' {
                i += 1;
                // Skip whitespace after colon
                while i < len && chars[i].is_whitespace() {
                    i += 1;
                }
                // Extract quoted string argument
                if i < len && chars[i] == '"' {
                    i += 1; // skip opening quote
                    let start = i;
                    // Find closing quote (handle escaped quotes)
                    while i < len && chars[i] != '"' {
                        if chars[i] == '\\' && i + 1 < len {
                            i += 2; // skip escaped char
                        } else {
                            i += 1;
                        }
                    }
                    let arg: String = chars[start..i].iter().collect();
                    // Replace `this.` with `ctx.` for the generated function
                    let converted_arg = arg.trim().replace("this.", "ctx.");
                    result.push_str(&converted_arg);
                    if i < len {
                        i += 1; // skip closing quote
                    }
                    continue;
                }
            }
        }
        i += 1;
    }

    result
}

/// Check if chars matches target starting at position
fn matches_at(chars: &[char], pos: usize, target: &[char]) -> bool {
    if pos + target.len() > chars.len() {
        return false;
    }
    for (j, &tc) in target.iter().enumerate() {
        if chars[pos + j] != tc {
            return false;
        }
    }
    true
}


/// Generate the JS module content
/// The generated template strips the SSR wrapper (<template shadowrootmode="open">)
/// since Lit renders directly into the shadow root on the client
pub fn generate_js_module(html_content: &str, lit_import: &str) -> String {
    // Strip the <template shadowrootmode="open"> wrapper if present
    let client_content = strip_template_wrapper(html_content);

    format!(
        r#"import {{ html }} from '{lit_import}';

/**
 * Returns the template for this component.
 * Use with LitSsrElement.renderTemplate() for CSR rendering.
 */
export function getTemplate(ctx) {{
  return html`{}`;
}}
"#,
        client_content.trim()
    )
}

/// Strip the <template shadowrootmode="..."> wrapper from content
/// This is needed because SSR uses declarative shadow DOM, but CSR renders directly
fn strip_template_wrapper(content: &str) -> String {
    let trimmed = content.trim();

    // Check if it starts with <template and ends with </template>
    if trimmed.starts_with("<template") && trimmed.ends_with("</template>") {
        // Find the end of the opening tag
        if let Some(open_end) = trimmed.find('>') {
            // Find the start of the closing tag
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

    #[test]
    fn test_extract_client_template_value() {
        let input = r#"<div>
  {{ firstName | capitalize | clientTemplateValue: "${this.firstName}" }}
  {{ lastName | clientTemplateValue: "${this.lastName}" }}
  {{ age | plus: 1 }}
</div>"#;

        // this. should be converted to ctx.
        let expected = "<div>\n  ${ctx.firstName}\n  ${ctx.lastName}\n  \n</div>";

        let result = process_liquid_content(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_client_template_values_in_one_expression() {
        let input = r#"{{ a | clientTemplateValue: "${this.a}" | clientTemplateValue: "${this.b}" }}"#;
        let result = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.a}${ctx.b}");
    }

    #[test]
    fn test_no_client_template_value() {
        let input = "{{ age | plus: 1 }}";
        let result = process_liquid_content(input).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_arbitrary_interpolation() {
        let input = r#"{{ items | clientTemplateValue: "${this.items.map(i => i.name).join(', ')}" }}"#;
        let result = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.items.map(i => i.name).join(', ')}");
    }

    #[test]
    fn test_complex_expression() {
        let input = r#"{{ x | clientTemplateValue: "${this.fn(a, b)}" }}"#;
        let result = process_liquid_content(input).unwrap();
        assert_eq!(result, "${ctx.fn(a, b)}");
    }
}
