//! Code generation for server-side render functions.
//!
//! Each [`TargetEmitter`] takes a [`ParsedTemplate`] and produces a source file
//! containing a typed render function — no Liquid engine required at runtime.

pub mod csharp;
pub use csharp::CSharpEmitter;

use crate::ParsedTemplate;

/// Emits a server-side render function from a [`ParsedTemplate`].
pub trait TargetEmitter: Send + Sync {
    /// Generate the complete source file content for this target.
    fn emit(&self, template_name: &str, parsed: &ParsedTemplate) -> String;
    /// File extension for the generated output (e.g., `"cs"`, `"go"`).
    fn file_extension(&self) -> &str;
}

/// Parse `--emit` flag value into a list of emitters.
///
/// `emit` is a comma-separated string like `"csharp"` or `"csharp,go"`.
/// Unknown targets are warned about and skipped.
pub fn build_emitters(emit: Option<&str>, namespace: &str) -> Vec<Box<dyn TargetEmitter>> {
    let Some(emit_str) = emit else {
        return vec![];
    };
    emit_str
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .filter_map(|target| match target {
            "csharp" => Some(
                Box::new(CSharpEmitter { namespace: namespace.to_string() })
                    as Box<dyn TargetEmitter>,
            ),
            other => {
                eprintln!(
                    "litquid warning: unknown emit target '{}', ignoring (supported: csharp)",
                    other
                );
                None
            }
        })
        .collect()
}
