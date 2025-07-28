use crate::search::Item;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn consolidate_directory(directory: &Path) -> io::Result<ConsolidationResult> {
    if !directory.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory not found: {}", directory.display()),
        ));
    }

    let mut results = ConsolidationResult::new();
    let mut dir_files: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let mut json_files = Vec::new();

            for subentry in fs::read_dir(&path)? {
                let subentry = subentry?;
                let subpath = subentry.path();

                if subpath.is_file() && subpath.extension().and_then(|s| s.to_str()) == Some("json")
                {
                    let filename = subpath.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if filename != "all.json" {
                        json_files.push(subpath);
                    }
                }
            }

            if !json_files.is_empty() {
                dir_files.insert(path, json_files);
            }
        }
    }

    for (dir_path, json_files) in dir_files {
        match consolidate_files(&dir_path, &json_files) {
            Ok(count) => {
                results.successful_dirs += 1;
                results.total_items_consolidated += count;
                results.directories_processed.push(DirectoryResult {
                    path: dir_path.clone(),
                    items_count: count,
                    error: None,
                });
            }
            Err(e) => {
                results.failed_dirs += 1;
                results.directories_processed.push(DirectoryResult {
                    path: dir_path.clone(),
                    items_count: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

fn consolidate_files(dir_path: &Path, json_files: &[PathBuf]) -> io::Result<usize> {
    let mut all_items = Vec::new();

    for json_file in json_files {
        let contents = fs::read_to_string(json_file)?;
        match serde_json::from_str::<Vec<Item>>(&contents) {
            Ok(items) => {
                all_items.extend(items);
            }
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse {}: {}", json_file.display(), e),
                ));
            }
        }
    }

    all_items.sort_by(|a, b| a.name.cmp(&b.name));

    let all_json_path = dir_path.join("all.json");
    let json_content = serde_json::to_string_pretty(&all_items)?;
    fs::write(&all_json_path, json_content)?;

    Ok(all_items.len())
}

#[derive(Debug)]
pub struct ConsolidationResult {
    pub successful_dirs: usize,
    pub failed_dirs: usize,
    pub total_items_consolidated: usize,
    pub directories_processed: Vec<DirectoryResult>,
}

#[derive(Debug)]
pub struct DirectoryResult {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub items_count: usize,
    pub error: Option<String>,
}

impl ConsolidationResult {
    fn new() -> Self {
        ConsolidationResult {
            successful_dirs: 0,
            failed_dirs: 0,
            total_items_consolidated: 0,
            directories_processed: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_items() -> Vec<Item> {
        vec![
            Item {
                name: "Test Item 1".to_string(),
                description: "First test item".to_string(),
                categories: Some(vec!["test".to_string()]),
                notes: None,
            },
            Item {
                name: "Test Item 2".to_string(),
                description: "Second test item".to_string(),
                categories: None,
                notes: Some("Test note".to_string()),
            },
        ]
    }

    #[test]
    fn test_consolidate_single_directory() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let items = create_test_items();
        fs::write(
            sub_dir.join("file1.json"),
            serde_json::to_string(&items[..1]).unwrap(),
        )
        .unwrap();
        fs::write(
            sub_dir.join("file2.json"),
            serde_json::to_string(&items[1..]).unwrap(),
        )
        .unwrap();

        let result = consolidate_directory(temp_dir.path()).unwrap();

        assert_eq!(result.successful_dirs, 1);
        assert_eq!(result.failed_dirs, 0);
        assert_eq!(result.total_items_consolidated, 2);

        let all_json_path = sub_dir.join("all.json");
        assert!(all_json_path.exists());

        let consolidated_content = fs::read_to_string(all_json_path).unwrap();
        let consolidated_items: Vec<Item> = serde_json::from_str(&consolidated_content).unwrap();
        assert_eq!(consolidated_items.len(), 2);
    }

    #[test]
    fn test_consolidate_multiple_directories() {
        let temp_dir = TempDir::new().unwrap();

        for i in 1..=3 {
            let sub_dir = temp_dir.path().join(format!("dir{}", i));
            fs::create_dir(&sub_dir).unwrap();

            let items = vec![Item {
                name: format!("Item {}", i),
                description: format!("Description {}", i),
                categories: None,
                notes: None,
            }];

            fs::write(
                sub_dir.join("data.json"),
                serde_json::to_string(&items).unwrap(),
            )
            .unwrap();
        }

        let result = consolidate_directory(temp_dir.path()).unwrap();

        assert_eq!(result.successful_dirs, 3);
        assert_eq!(result.failed_dirs, 0);
        assert_eq!(result.total_items_consolidated, 3);
    }

    #[test]
    fn test_consolidate_skips_all_json() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let items = create_test_items();
        fs::write(
            sub_dir.join("data.json"),
            serde_json::to_string(&items).unwrap(),
        )
        .unwrap();
        fs::write(
            sub_dir.join("all.json"),
            serde_json::to_string(&items).unwrap(),
        )
        .unwrap();

        let result = consolidate_directory(temp_dir.path()).unwrap();

        assert_eq!(result.successful_dirs, 1);
        assert_eq!(result.total_items_consolidated, 2);
    }

    #[test]
    fn test_consolidate_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let result = consolidate_directory(temp_dir.path()).unwrap();

        assert_eq!(result.successful_dirs, 0);
        assert_eq!(result.failed_dirs, 0);
        assert_eq!(result.total_items_consolidated, 0);
    }

    #[test]
    fn test_consolidate_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        fs::write(sub_dir.join("invalid.json"), "not valid json").unwrap();

        let result = consolidate_directory(temp_dir.path()).unwrap();

        assert_eq!(result.successful_dirs, 0);
        assert_eq!(result.failed_dirs, 1);
        assert!(result.directories_processed[0].error.is_some());
    }

    #[test]
    fn test_consolidate_nonexistent_directory() {
        let result = consolidate_directory(Path::new("/nonexistent/directory"));
        assert!(result.is_err());
    }

    #[test]
    fn test_consolidate_sorts_items() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let items = vec![
            Item {
                name: "Zebra".to_string(),
                description: "Last alphabetically".to_string(),
                categories: None,
                notes: None,
            },
            Item {
                name: "Apple".to_string(),
                description: "First alphabetically".to_string(),
                categories: None,
                notes: None,
            },
        ];

        fs::write(
            sub_dir.join("data.json"),
            serde_json::to_string(&items).unwrap(),
        )
        .unwrap();

        consolidate_directory(temp_dir.path()).unwrap();

        let all_json_content = fs::read_to_string(sub_dir.join("all.json")).unwrap();
        let consolidated: Vec<Item> = serde_json::from_str(&all_json_content).unwrap();

        assert_eq!(consolidated[0].name, "Apple");
        assert_eq!(consolidated[1].name, "Zebra");
    }
}
