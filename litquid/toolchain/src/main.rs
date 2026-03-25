use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use litquid::{codegen::build_emitters, process_liquid_file, DEFAULT_LIT_IMPORT};

/// LitQuid Preprocessor - Converts .liquid templates to Lit template modules
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing .liquid files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated files
    #[arg(short, long)]
    output: PathBuf,

    /// Import path for Lit (e.g., "lit", "https://cdn.jsdelivr.net/npm/lit@3/+esm")
    #[arg(long, default_value = DEFAULT_LIT_IMPORT)]
    lit_import: String,

    /// Additional server-side render targets (comma-separated). Supported: csharp
    ///
    /// Example: --emit csharp
    /// Generates a .template.cs alongside each .template.js with a typed Render()
    /// method — no Liquid engine required at runtime.
    #[arg(long)]
    emit: Option<String>,

    /// C# namespace for generated code (used with --emit csharp)
    #[arg(long, default_value = "LitQuid.Generated")]
    namespace: String,
}

fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: Input directory does not exist: {:?}", args.input);
        std::process::exit(1);
    }

    fs::create_dir_all(&args.output).expect("Failed to create output directory");

    let emitters = build_emitters(args.emit.as_deref(), &args.namespace);

    for entry in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "liquid"))
    {
        let input_path = entry.path();
        let relative_path = input_path.strip_prefix(&args.input).unwrap();
        let template_name = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template");

        let output_js_path = args
            .output
            .join(relative_path)
            .with_extension("template.js");
        let output_json_path = args
            .output
            .join(relative_path)
            .with_extension("template.json");

        if let Some(parent) = output_js_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create output subdirectory");
        }

        match process_liquid_file(input_path, Some(&args.lit_import)) {
            Ok(parsed) => {
                fs::write(&output_js_path, parsed.to_js_module())
                    .expect("Failed to write JS file");
                fs::write(&output_json_path, parsed.to_json_manifest())
                    .expect("Failed to write manifest file");
                println!("Generated: {:?}", output_js_path);

                for emitter in &emitters {
                    let content = emitter.emit(template_name, &parsed);
                    let ext = format!("template.{}", emitter.file_extension());
                    let output_path =
                        args.output.join(relative_path).with_extension(&ext);
                    fs::write(&output_path, content)
                        .expect("Failed to write generated file");
                    println!("Generated: {:?}", output_path);
                }
            }
            Err(e) => {
                eprintln!("Error processing {:?}: {}", input_path, e);
            }
        }
    }
}
