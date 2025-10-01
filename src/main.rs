use clap::Parser;
use color_eyre::Result;
use netbook::cli::{CliArgs, run_cli};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = CliArgs::parse();
    run_cli(args).await
}
