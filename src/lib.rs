pub mod consolidate;
pub mod search;
pub mod validator;

use clap::Subcommand;
use std::path::PathBuf;

use consolidate::consolidate_directory;
use search::{SearchOptions, search_directory};
use validator::validate_directory;

#[derive(Subcommand)]
pub enum Commands {
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
pub struct CommandResult {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub exit_code: i32,
}

pub fn run_command(command: Commands) -> CommandResult {
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

                let successful_msg =
                    format!("  Successful directories: {}", result.successful_dirs);
                output.push(successful_msg);

                let failed_msg = format!("  Failed directories: {}", result.failed_dirs);
                output.push(failed_msg);

                let total_msg = format!(
                    "  Total items consolidated: {}",
                    result.total_items_consolidated
                );
                output.push(total_msg);

                if result.failed_dirs > 0 {
                    output.push("".to_string());
                    output.push("Errors:".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_run_command_validate_no_errors() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let valid_content = r#"[{"name": "Item1", "description": "Test1"}]"#;
        fs::write(sub_dir.join("data.json"), valid_content).unwrap();

        let result = run_command(Commands::Validate {
            directory: temp_dir.path().to_path_buf(),
        });

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_validate_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Missing required "description" field
        let invalid_content = r#"[{"name": "Item without description"}]"#;
        fs::write(sub_dir.join("invalid.json"), invalid_content).unwrap();

        let result = run_command(Commands::Validate {
            directory: temp_dir.path().to_path_buf(),
        });

        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_validate_error() {
        let result = run_command(Commands::Validate {
            directory: PathBuf::from("/nonexistent/directory"),
        });

        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr.len(), 1);
        assert!(result.stderr[0].contains("Error:"));
    }

    #[test]
    fn test_run_command_search_found() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("items.json");

        let items = vec![
            serde_json::json!({
                "name": "Red Pen",
                "description": "A red ballpoint pen",
                "categories": ["stationery"],
            }),
            serde_json::json!({
                "name": "Blue Notebook",
                "description": "A blue spiral notebook",
                "categories": ["stationery"],
                "notes": "For meetings",
            }),
        ];

        fs::write(&json_path, serde_json::to_string(&items).unwrap()).unwrap();

        let result = run_command(Commands::Search {
            directory: temp_dir.path().to_path_buf(),
            keywords: vec!["notebook".to_string()],
            name: false,
            description: false,
            categories: false,
            notes: false,
            all: true,
        });

        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.is_empty());
        assert_eq!(result.stdout[0], "Found 1 items:");
        assert!(result.stdout.contains(&"Name: Blue Notebook".to_string()));
        assert!(
            result
                .stdout
                .contains(&"Description: A blue spiral notebook".to_string())
        );
        assert!(
            result
                .stdout
                .contains(&"Categories: stationery".to_string())
        );
        assert!(result.stdout.contains(&"Notes: For meetings".to_string()));
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_search_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("items.json");

        let items = vec![serde_json::json!({
            "name": "Red Pen",
            "description": "A red ballpoint pen",
        })];

        fs::write(&json_path, serde_json::to_string(&items).unwrap()).unwrap();

        let result = run_command(Commands::Search {
            directory: temp_dir.path().to_path_buf(),
            keywords: vec!["notebook".to_string()],
            name: false,
            description: false,
            categories: false,
            notes: false,
            all: true,
        });

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.len(), 1);
        assert_eq!(result.stdout[0], "No items found matching all keywords.");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_search_error() {
        let result = run_command(Commands::Search {
            directory: PathBuf::from("/nonexistent/directory"),
            keywords: vec!["test".to_string()],
            name: false,
            description: false,
            categories: false,
            notes: false,
            all: true,
        });

        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr.len(), 1);
        assert!(result.stderr[0].contains("Error searching:"));
    }

    #[test]
    fn test_run_command_consolidate_success() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let items = vec![serde_json::json!({
            "name": "Item1",
            "description": "Description1",
        })];

        fs::write(
            sub_dir.join("data.json"),
            serde_json::to_string(&items).unwrap(),
        )
        .unwrap();

        let result = run_command(Commands::Consolidate {
            directory: temp_dir.path().to_path_buf(),
        });

        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.is_empty());
        assert_eq!(result.stdout[0], "Consolidation complete:");
        assert_eq!(result.stdout[1], "  Successful directories: 1");
        assert_eq!(result.stdout[2], "  Failed directories: 0");
        assert_eq!(result.stdout[3], "  Total items consolidated: 1");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_consolidate_with_failures() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Create invalid JSON file
        fs::write(sub_dir.join("invalid.json"), "not valid json").unwrap();

        let result = run_command(Commands::Consolidate {
            directory: temp_dir.path().to_path_buf(),
        });

        assert_eq!(result.exit_code, 1);
        assert!(!result.stdout.is_empty());
        assert_eq!(result.stdout[0], "Consolidation complete:");
        assert_eq!(result.stdout[1], "  Successful directories: 0");
        assert_eq!(result.stdout[2], "  Failed directories: 1");
        assert_eq!(result.stdout[3], "  Total items consolidated: 0");
        assert_eq!(result.stdout[4], "");
        assert_eq!(result.stdout[5], "Errors:");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_run_command_consolidate_error() {
        let result = run_command(Commands::Consolidate {
            directory: PathBuf::from("/nonexistent/directory"),
        });

        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr.len(), 1);
        assert!(result.stderr[0].contains("Error:"));
    }

    #[test]
    fn test_run_command_consolidate_mixed_results() {
        let temp_dir = TempDir::new().unwrap();

        // Create two subdirectories
        let sub_dir1 = temp_dir.path().join("subdir1");
        let sub_dir2 = temp_dir.path().join("subdir2");
        fs::create_dir(&sub_dir1).unwrap();
        fs::create_dir(&sub_dir2).unwrap();

        // Valid JSON in first directory
        let items = vec![serde_json::json!({
            "name": "Item1",
            "description": "Description1",
        })];
        fs::write(
            sub_dir1.join("data.json"),
            serde_json::to_string(&items).unwrap(),
        )
        .unwrap();

        // Invalid JSON in second directory
        fs::write(sub_dir2.join("invalid.json"), "not valid json").unwrap();

        let result = run_command(Commands::Consolidate {
            directory: temp_dir.path().to_path_buf(),
        });

        assert_eq!(result.exit_code, 1);
        assert!(!result.stdout.is_empty());

        // Verify the exact output format to ensure coverage of format! macros
        assert_eq!(result.stdout[0], "Consolidation complete:");
        assert_eq!(result.stdout[1], "  Successful directories: 1");
        assert_eq!(result.stdout[2], "  Failed directories: 1");
        assert_eq!(result.stdout[3], "  Total items consolidated: 1");
        assert_eq!(result.stdout[4], "");
        assert_eq!(result.stdout[5], "Errors:");

        assert!(result.stderr.is_empty());
    }
}
