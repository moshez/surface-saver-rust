use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod validator;

use validator::validate_directory;

#[derive(Parser)]
#[command(name = "surface-saver")]
#[command(about = "A CLI app with subcommands", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a directory
    Validate {
        /// The directory to validate
        #[arg(value_name = "DIR")]
        directory: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { directory } => {
            if let Err(e) = validate_directory(&directory) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}