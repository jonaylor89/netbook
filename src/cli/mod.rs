use color_eyre::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "netbook")]
#[command(about = "A lightweight TUI request collection manager and runner")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to collection file
    #[arg(short, long)]
    pub collection: Option<PathBuf>,
}

#[derive(Parser)]
pub enum Commands {
    /// Open TUI interface (default)
    Open {
        /// Collection file path
        collection: PathBuf,
    },
    /// Run request in headless mode
    #[command(name = "headless-run")]
    HeadlessRun {
        /// Name of request to run
        name: String,
        /// Collection file path
        #[arg(short, long)]
        collection: PathBuf,
    },
    /// Export last response
    Export {
        /// Export file path
        path: PathBuf,
    },
}

pub async fn run_cli(args: CliArgs) -> Result<()> {
    match args.command {
        Some(Commands::Open { collection }) => {
            crate::tui::run_tui(collection).await
        }
        Some(Commands::HeadlessRun { name, collection }) => {
            crate::core::run_headless(&name, &collection).await
        }
        Some(Commands::Export { path }) => {
            crate::io::export_last_response(&path).await
        }
        None => {
            if let Some(collection) = args.collection {
                crate::tui::run_tui(collection).await
            } else {
                eprintln!("Error: Collection file is required");
                std::process::exit(1);
            }
        }
    }
}