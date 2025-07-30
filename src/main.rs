use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod consolidate;
mod search;
mod validator;

use consolidate::consolidate_directory;
use search::{SearchOptions, search_directory};
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
    /// Consolidate JSON files in each subdirectory into all.json
    Consolidate {
        /// The directory to consolidate
        #[arg(value_name = "DIR")]
        directory: PathBuf,
    },
}

#[derive(Debug)]
struct CommandResult {
    stdout: Vec<String>,
    stderr: Vec<String>,
    exit_code: i32,
}

fn run_command(command: Commands) -> CommandResult {
    match command {
        Commands::Validate { directory } => match validate_directory(&directory) {
            Ok(has_errors) => {
                if has_errors {
                    CommandResult {
                        stdout: vec![],
                        stderr: vec![],
                        exit_code: 1,
                    }
                } else {
                    CommandResult {
                        stdout: vec![],
                        stderr: vec![],
                        exit_code: 0,
                    }
                }
            }
            Err(e) => CommandResult {
                stdout: vec![],
                stderr: vec![format!("Error: {e}")],
                exit_code: 1,
            },
        },
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
                    let mut output = Vec::new();
                    if results.is_empty() {
                        output.push("No items found matching all keywords.".to_string());
                    } else {
                        output.push(format!("Found {} items:", results.len()));
                        for result in results {
                            output.push("\n---".to_string());
                            output.push(format!("Name: {}", result.item.name));
                            output.push(format!("Description: {}", result.item.description));
                            if let Some(cats) = &result.item.categories {
                                output.push(format!("Categories: {}", cats.join(", ")));
                            }
                            if let Some(notes) = &result.item.notes {
                                output.push(format!("Notes: {notes}"));
                            }
                            output.push(format!("File: {}", result.file_path.display()));
                        }
                    }
                    CommandResult {
                        stdout: output,
                        stderr: vec![],
                        exit_code: 0,
                    }
                }
                Err(e) => CommandResult {
                    stdout: vec![],
                    stderr: vec![format!("Error searching: {e}")],
                    exit_code: 1,
                },
            }
        }
        Commands::Consolidate { directory } => match consolidate_directory(&directory) {
            Ok(result) => {
                let mut output = Vec::new();
                output.push("Consolidation complete:".to_string());
                output.push(format!(
                    "  Successful directories: {}",
                    result.successful_dirs
                ));
                output.push(format!("  Failed directories: {}", result.failed_dirs));
                output.push(format!(
                    "  Total items consolidated: {}",
                    result.total_items_consolidated
                ));

                if result.failed_dirs > 0 {
                    output.push("\nErrors:".to_string());
                    for dir_result in &result.directories_processed {
                        if let Some(error) = &dir_result.error {
                            output.push(format!("  {}: {}", dir_result.path.display(), error));
                        }
                    }
                    CommandResult {
                        stdout: output,
                        stderr: vec![],
                        exit_code: 1,
                    }
                } else {
                    CommandResult {
                        stdout: output,
                        stderr: vec![],
                        exit_code: 0,
                    }
                }
            }
            Err(e) => CommandResult {
                stdout: vec![],
                stderr: vec![format!("Error: {e}")],
                exit_code: 1,
            },
        },
    }
}

fn main() {
    let cli = Cli::parse();

    let result = run_command(cli.command);

    for line in result.stdout {
        println!("{line}");
    }

    for line in result.stderr {
        eprintln!("{line}");
    }

    if result.exit_code != 0 {
        std::process::exit(result.exit_code);
    }
}
