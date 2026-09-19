use clap::Parser;
use hypermemory::cli::{run_cli, Cli};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    run_cli(cli).await
}
