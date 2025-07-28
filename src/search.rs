use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    pub name: String,
    pub description: String,
    pub categories: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub item: Item,
    pub file_path: PathBuf,
}

pub struct SearchOptions {
    pub search_name: bool,
    pub search_description: bool,
    pub search_categories: bool,
    pub search_notes: bool,
    pub search_all: bool,
}

impl SearchOptions {
    pub fn should_search_name(&self) -> bool {
        self.search_name
            || self.search_all
            || (!self.search_description && !self.search_categories && !self.search_notes)
    }

    pub fn should_search_description(&self) -> bool {
        self.search_description
            || self.search_all
            || (!self.search_name && !self.search_categories && !self.search_notes)
    }

    pub fn should_search_categories(&self) -> bool {
        self.search_categories
            || self.search_all
            || (!self.search_name && !self.search_description && !self.search_notes)
    }

    pub fn should_search_notes(&self) -> bool {
        self.search_notes
            || self.search_all
            || (!self.search_name && !self.search_description && !self.search_categories)
    }
}

pub fn search_directory(
    directory: &Path,
    keywords: &[String],
    options: &SearchOptions,
) -> io::Result<Vec<SearchResult>> {
    if !directory.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory not found: {}", directory.display()),
        ));
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(items) = serde_json::from_str::<Vec<Item>>(&contents) {
                    for item in items {
                        if matches_all_keywords(&item, keywords, options) {
                            results.push(SearchResult {
                                item,
                                file_path: path.to_path_buf(),
                            });
                        }
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.item.name.cmp(&b.item.name));
    Ok(results)
}

fn matches_all_keywords(item: &Item, keywords: &[String], options: &SearchOptions) -> bool {
    for keyword in keywords {
        if !matches_keyword(item, keyword, options) {
            return false;
        }
    }
    true
}

fn matches_keyword(item: &Item, keyword: &str, options: &SearchOptions) -> bool {
    let keyword_lower = keyword.to_lowercase();

    if options.should_search_name() && item.name.to_lowercase().contains(&keyword_lower) {
        return true;
    }

    if options.should_search_description()
        && item.description.to_lowercase().contains(&keyword_lower)
    {
        return true;
    }

    if options.should_search_categories() {
        if let Some(categories) = &item.categories {
            for category in categories {
                if category.to_lowercase().contains(&keyword_lower) {
                    return true;
                }
            }
        }
    }

    if options.should_search_notes() {
        if let Some(notes) = &item.notes {
            if notes.to_lowercase().contains(&keyword_lower) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_item() -> Item {
        Item {
            name: "Test Notebook".to_string(),
            description: "A red spiral notebook".to_string(),
            categories: Some(vec!["stationery".to_string(), "notebooks".to_string()]),
            notes: Some("Important notes here".to_string()),
        }
    }

    #[test]
    fn test_search_options_default_behavior() {
        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: false,
            search_notes: false,
            search_all: false,
        };

        assert!(options.should_search_name());
        assert!(options.should_search_description());
        assert!(options.should_search_categories());
        assert!(options.should_search_notes());
    }

    #[test]
    fn test_search_options_all_flag() {
        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: false,
            search_notes: false,
            search_all: true,
        };

        assert!(options.should_search_name());
        assert!(options.should_search_description());
        assert!(options.should_search_categories());
        assert!(options.should_search_notes());
    }

    #[test]
    fn test_search_options_specific_fields() {
        let options = SearchOptions {
            search_name: true,
            search_description: false,
            search_categories: true,
            search_notes: false,
            search_all: false,
        };

        assert!(options.should_search_name());
        assert!(!options.should_search_description());
        assert!(options.should_search_categories());
        assert!(!options.should_search_notes());
    }

    #[test]
    fn test_matches_keyword_in_name() {
        let item = create_test_item();
        let options = SearchOptions {
            search_name: true,
            search_description: false,
            search_categories: false,
            search_notes: false,
            search_all: false,
        };

        assert!(matches_keyword(&item, "notebook", &options));
        assert!(matches_keyword(&item, "NOTEBOOK", &options)); // Case insensitive
        assert!(!matches_keyword(&item, "red", &options)); // Not in name
    }

    #[test]
    fn test_matches_keyword_in_description() {
        let item = create_test_item();
        let options = SearchOptions {
            search_name: false,
            search_description: true,
            search_categories: false,
            search_notes: false,
            search_all: false,
        };

        assert!(matches_keyword(&item, "red", &options));
        assert!(matches_keyword(&item, "spiral", &options));
        assert!(!matches_keyword(&item, "blue", &options));
    }

    #[test]
    fn test_matches_keyword_in_categories() {
        let item = create_test_item();
        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: true,
            search_notes: false,
            search_all: false,
        };

        assert!(matches_keyword(&item, "stationery", &options));
        assert!(matches_keyword(&item, "notebooks", &options));
        assert!(!matches_keyword(&item, "electronics", &options));
    }

    #[test]
    fn test_matches_keyword_in_notes() {
        let item = create_test_item();
        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: false,
            search_notes: true,
            search_all: false,
        };

        assert!(matches_keyword(&item, "important", &options));
        assert!(matches_keyword(&item, "notes", &options));
        assert!(!matches_keyword(&item, "trivial", &options));
    }

    #[test]
    fn test_matches_all_keywords() {
        let item = create_test_item();
        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: false,
            search_notes: false,
            search_all: true,
        };

        let keywords = vec!["notebook".to_string(), "red".to_string()];
        assert!(matches_all_keywords(&item, &keywords, &options));

        let keywords = vec!["notebook".to_string(), "blue".to_string()];
        assert!(!matches_all_keywords(&item, &keywords, &options));
    }

    #[test]
    fn test_search_directory() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("items.json");

        let items = vec![
            Item {
                name: "Red Pen".to_string(),
                description: "A red ballpoint pen".to_string(),
                categories: Some(vec!["stationery".to_string()]),
                notes: None,
            },
            Item {
                name: "Blue Notebook".to_string(),
                description: "A blue spiral notebook".to_string(),
                categories: Some(vec!["stationery".to_string()]),
                notes: Some("For meetings".to_string()),
            },
        ];

        fs::write(&json_path, serde_json::to_string(&items).unwrap()).unwrap();

        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: false,
            search_notes: false,
            search_all: true,
        };

        let results =
            search_directory(temp_dir.path(), &["stationery".to_string()], &options).unwrap();
        assert_eq!(results.len(), 2);

        let results =
            search_directory(temp_dir.path(), &["notebook".to_string()], &options).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.name, "Blue Notebook");

        let results = search_directory(temp_dir.path(), &["red".to_string()], &options).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.name, "Red Pen");
    }

    #[test]
    fn test_search_with_empty_optional_fields() {
        let item = Item {
            name: "Simple Item".to_string(),
            description: "Just a description".to_string(),
            categories: None,
            notes: None,
        };

        let options = SearchOptions {
            search_name: false,
            search_description: false,
            search_categories: true,
            search_notes: true,
            search_all: false,
        };

        assert!(!matches_keyword(&item, "anything", &options));
    }
}
