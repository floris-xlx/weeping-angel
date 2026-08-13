use clap::Parser;
use tracing_subscriber::EnvFilter;
use weeping_angel::cli::{Cli, Commands};
use weeping_angel::{
    run_finalize_command, run_scan_code_command, run_scan_diff_command, run_scan_command,
    run_workbench_command,
};
use weeping_angel::style;

#[tokio::main]
async fn main() {
    style::init();
    let cli: Cli = Cli::parse();

    // Default: keep tracing quieter so live request/ANSI lines stay readable.
    // Use --verbose / --verbose --verbose or RUST_LOG for engine debug noise.
    let filter: String = match cli.verbose {
        0 => "weeping_angel=warn".to_string(),
        1 => "weeping_angel=info".to_string(),
        _ => "weeping_angel=debug,debug".to_string(),
    };
    let filter: EnvFilter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(style::color_enabled())
        .init();

    let code: i32 = match cli.command {
        Commands::Scan(args) => match run_scan_command(args).await {
            Ok(code) => code,
            Err(e) => {
                style::eprint_line(&format!("{} {e:#}", style::err("error:")));
                2
            }
        },
        Commands::Finalize(args) => match run_finalize_command(args) {
            Ok(code) => code,
            Err(e) => {
                style::eprint_line(&format!("{} {e:#}", style::err("error:")));
                2
            }
        },
        Commands::ScanCode(args) => match run_scan_code_command(args) {
            Ok(code) => code,
            Err(e) => {
                style::eprint_line(&format!("{} {e:#}", style::err("error:")));
                2
            }
        },
        Commands::ScanDiff(args) => match run_scan_diff_command(args) {
            Ok(code) => code,
            Err(e) => {
                style::eprint_line(&format!("{} {e:#}", style::err("error:")));
                2
            }
        },
        Commands::Workbench(args) => match run_workbench_command(args) {
            Ok(code) => code,
            Err(e) => {
                style::eprint_line(&format!("{} {e:#}", style::err("error:")));
                2
            }
        },
        Commands::Version => {
            println!("{}", weeping_angel::cli::VERSION_LINE);
            0
        }
    };

    std::process::exit(code);
}
