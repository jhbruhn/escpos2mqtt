/// CLI tool to generate DSL documentation
/// Documentation is automatically extracted from parser annotations
use escpos2mqtt::program::{documentation, Command};
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Trigger parser loading to ensure all commands are registered
    let _ = Command::parse("");

    let args: Vec<String> = std::env::args().collect();

    let format = args.get(1).map(|s| s.as_str()).unwrap_or("markdown");
    let output_path = args.get(2).map(PathBuf::from);

    let content = match format {
        "markdown" | "md" => documentation::generate_markdown(),
        "text" | "txt" => documentation::generate_text(),
        _ => {
            eprintln!("Usage: generate_docs [markdown|text] [output_file]");
            eprintln!("Formats: markdown (default), text");
            eprintln!("If output_file is not specified, prints to stdout");
            std::process::exit(1);
        }
    };

    if let Some(path) = output_path {
        fs::write(&path, content)?;
        println!("Documentation written to: {}", path.display());
    } else {
        print!("{}", content);
    }

    Ok(())
}
