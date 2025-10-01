use color_eyre::Result;
use netbook::cli::{run_cli, CliArgs};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = CliArgs::parse();
    run_cli(args).await
}
