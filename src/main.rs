use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod validator;
mod search;

use validator::validate_directory;
use search::{search_directory, SearchOptions};

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
    /// Search for items by keywords
    Search {
        /// The directory to search in
        directory: PathBuf,
        
        /// Keywords to search for (all must match)
        keywords: Vec<String>,
        
        /// Search only in name field
        #[arg(long = "name")]
        name: bool,
        
        /// Search only in description field
        #[arg(long = "description")]
        description: bool,
        
        /// Search only in categories
        #[arg(long = "categories")]
        categories: bool,
        
        /// Search only in notes field
        #[arg(long = "notes")]
        notes: bool,
        
        /// Search in all fields (default if no specific field is selected)
        #[arg(long = "all")]
        all: bool,
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
        Commands::Search {
            directory,
            keywords,
            name,
            description,
            categories,
            notes,
            all,
        } => {
            let options = SearchOptions {
                search_name: name,
                search_description: description,
                search_categories: categories,
                search_notes: notes,
                search_all: all,
            };

            match search_directory(&directory, &keywords, &options) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No items found matching all keywords.");
                    } else {
                        println!("Found {} items:", results.len());
                        for result in results {
                            println!("\n---");
                            println!("Name: {}", result.item.name);
                            println!("Description: {}", result.item.description);
                            if let Some(cats) = &result.item.categories {
                                println!("Categories: {}", cats.join(", "));
                            }
                            if let Some(notes) = &result.item.notes {
                                println!("Notes: {}", notes);
                            }
                            println!("File: {}", result.file_path.display());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error searching: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}