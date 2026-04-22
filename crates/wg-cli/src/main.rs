mod cmd;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wg-cli", about = "WallGuard command-line interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Enroll this device with a WallGuard server.
    Enroll(cmd::enroll::EnrollArgs),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Command::Enroll(args) => cmd::enroll::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
