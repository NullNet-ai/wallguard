use clap::Parser;

mod cli;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let args = match cli::Args::try_parse() {
        Ok(args) => args,
        Err(err) => return log::error!("Failed to parse CLI arguments: {}", err),
    };

    if let Err(err) = args.validate() {
        return log::error!("{}", err);
    }

    log::info!("Execution complete");
}
