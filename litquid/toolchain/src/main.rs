use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use litquid::{process_liquid_file, DEFAULT_LIT_IMPORT};

/// LitQuid Preprocessor - Converts .liquid templates to Lit template modules
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing .liquid files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated .template.js files
    #[arg(short, long)]
    output: PathBuf,

    /// Import path for Lit (e.g., "lit", "https://cdn.jsdelivr.net/npm/lit@3/+esm")
    #[arg(long, default_value = DEFAULT_LIT_IMPORT)]
    lit_import: String,
}

fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: Input directory does not exist: {:?}", args.input);
        std::process::exit(1);
    }

    fs::create_dir_all(&args.output).expect("Failed to create output directory");

    for entry in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "liquid"))
    {
        let input_path = entry.path();
        let relative_path = input_path.strip_prefix(&args.input).unwrap();
        let output_path = args
            .output
            .join(relative_path)
            .with_extension("template.js");

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create output subdirectory");
        }

        match process_liquid_file(input_path, Some(&args.lit_import)) {
            Ok(js_content) => {
                fs::write(&output_path, js_content).expect("Failed to write output file");
                println!("Generated: {:?}", output_path);
            }
            Err(e) => {
                eprintln!("Error processing {:?}: {}", input_path, e);
            }
        }
    }
}

