use clap::Parser;
use tracing_subscriber::EnvFilter;
use weeping_angel::cli::{Cli, Commands};
use weeping_angel::run_scan_command;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "weeping_angel=info,warn".to_string(),
        1 => "weeping_angel=debug".to_string(),
        _ => "weeping_angel=trace,debug".to_string(),
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let code = match cli.command {
        Commands::Scan(args) => match run_scan_command(args).await {
            Ok(code) => code,
            Err(e) => {
                eprintln!("error: {e:#}");
                2
            }
        },
    };

    std::process::exit(code);
}
