use std::{fs, path::PathBuf, process::ExitCode};

use clap::Parser;
use weeping_angel::docs_export::export_command_reference;

#[derive(Parser, Debug)]
#[command(name = "weeping-angel-docs-export")]
struct ExportArgs {
    /// Command path to export, for example: scan
    path: Vec<String>,
    /// Write the export to a file instead of stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = ExportArgs::parse();

    let export = match export_command_reference(&args.path) {
        Ok(export) => export,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };

    let rendered = match serde_json::to_string_pretty(&export) {
        Ok(rendered) => rendered,
        Err(error) => {
            eprintln!("Failed to render command export as JSON: {error}");
            return ExitCode::from(1);
        }
    };

    if let Some(output) = args.output {
        if let Some(parent) = output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("Failed to create {}: {error}", parent.display());
                return ExitCode::from(1);
            }
        }
        if let Err(error) = fs::write(&output, format!("{rendered}\n")) {
            eprintln!("Failed to write {}: {error}", output.display());
            return ExitCode::from(1);
        }
    } else {
        println!("{rendered}");
    }

    ExitCode::SUCCESS
}
