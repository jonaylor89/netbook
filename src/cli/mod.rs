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
        /// Collection file path (optional, will auto-discover)
        collection: Option<PathBuf>,
    },
    /// Initialize a new collection in the current directory
    Init,
    /// Run request in headless mode
    #[command(name = "run")]
    HeadlessRun {
        /// Name of request to run
        name: String,
        /// Collection file path (optional, will auto-discover)
        #[arg(short, long)]
        collection: Option<PathBuf>,
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
            let collection_path = resolve_collection(collection)?;
            crate::tui::run_tui(collection_path).await
        }
        Some(Commands::Init) => {
            init_collection().await
        }
        Some(Commands::HeadlessRun { name, collection }) => {
            let collection_path = resolve_collection(collection)?;
            crate::core::run_headless(&name, &collection_path).await
        }
        Some(Commands::Export { path }) => {
            crate::io::export_last_response(&path).await
        }
        None => {
            let collection_path = resolve_collection(args.collection)?;
            crate::tui::run_tui(collection_path).await
        }
    }
}

fn resolve_collection(collection: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = collection {
        Ok(path)
    } else {
        let discovered = crate::io::discover_collection()?;

        // If collection doesn't exist, auto-init
        if !discovered.exists() {
            println!("No collection found. Creating one at: {}", discovered.display());
            crate::io::create_initial_collection(&discovered)?;
            println!("✓ Collection created with example requests\n");
        }

        Ok(discovered)
    }
}

async fn init_collection() -> Result<()> {
    let collection_path = crate::io::discover_collection()?;

    if collection_path.exists() {
        eprintln!("Collection already exists at: {}", collection_path.display());
        eprintln!("Use 'netbook open' to open it or delete it first.");
        std::process::exit(1);
    }

    crate::io::create_initial_collection(&collection_path)?;
    println!("✓ Created new collection at: {}", collection_path.display());
    println!("\nNext steps:");
    println!("  netbook open          # Open the TUI");
    println!("  $EDITOR {}  # Edit requests directly", collection_path.display());

    Ok(())
}
