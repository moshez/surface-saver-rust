use crate::search::{SearchOptions, search_directory};
use anyhow::Result;
use chrono::Local;
use rmcp::{
    ServerHandler,
    model::{ServerCapabilities, ServerInfo},
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRequest {
    #[schemars(description = "Keywords to search for (all must match)")]
    pub keywords: Vec<String>,
    #[schemars(description = "Search only in name field")]
    pub name: Option<bool>,
    #[schemars(description = "Search only in description field")]
    pub description: Option<bool>,
    #[schemars(description = "Search only in categories")]
    pub categories: Option<bool>,
    #[schemars(description = "Search only in notes field")]
    pub notes: Option<bool>,
    #[schemars(description = "Search in all fields (default if no specific field is selected)")]
    pub all: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubmitRequest {
    #[schemars(description = "The subdirectory name where the JSON file will be created")]
    pub name: String,
    #[schemars(
        description = "Array of items to add to the inventory. Each item must have 'name' and 'description' fields, with optional 'categories' (array of strings) and 'notes' (string) fields"
    )]
    pub items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct McpServer {
    pub(crate) directory: Arc<PathBuf>,
}

impl McpServer {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Arc::new(directory),
        }
    }
}

#[tool(tool_box)]
impl McpServer {
    #[tool(
        description = "Search for items in the inventory by keywords. Surface Saver helps catalog the inevitable accumulation of items on horizontal surfaces (desks, counters, shelves) into searchable JSON inventories. You can search by keywords across all fields or target specific fields like name, description, categories, or notes. Multiple keywords use AND logic - all must match. Examples: search for 'notebook' to find all notebooks, use name=true to search only item names, or combine keywords like 'arduino sensor' to find items matching both terms."
    )]
    async fn search(&self, #[tool(aggr)] request: SearchRequest) -> String {
        tracing::info!("Search request: {:?}", request);

        let options = SearchOptions {
            search_name: request.name.unwrap_or(false),
            search_description: request.description.unwrap_or(false),
            search_categories: request.categories.unwrap_or(false),
            search_notes: request.notes.unwrap_or(false),
            search_all: request.all.unwrap_or(false),
        };

        match search_directory(&self.directory, &request.keywords, &options) {
            Ok(results) => {
                if results.is_empty() {
                    "No items found matching all keywords.".to_string()
                } else {
                    let mut output = vec![format!("Found {} items:", results.len())];
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
                    output.join("\n")
                }
            }
            Err(e) => {
                tracing::error!("Search error: {}", e);
                format!("Error searching: {e}")
            }
        }
    }

    #[tool(
        description = "Submit new items to the inventory by creating a timestamped JSON file. Surface Saver helps catalog items on horizontal surfaces (desks, counters, shelves) into searchable inventories. When you submit items, they are saved to a new JSON file with a timestamp (e.g., 2025-07-29-08-30-22.json) in the specified subdirectory. Each item must have a 'name' and 'description', and can optionally include 'categories' (array of strings) and 'notes' (string). Example: submit to 'desk' directory with items [{\"name\": \"Arduino Uno\", \"description\": \"Microcontroller board with USB cable\", \"categories\": [\"electronics\", \"arduino\"], \"notes\": \"In project box\"}]"
    )]
    async fn submit(&self, #[tool(aggr)] request: SubmitRequest) -> String {
        tracing::info!("Submit request: {:?}", request);

        // Validate subdirectory name
        if request.name.is_empty() || request.name.contains('/') || request.name.contains('\\') {
            return "Error: Invalid subdirectory name. Name must be non-empty and cannot contain path separators.".to_string();
        }

        // Validate items
        if request.items.is_empty() {
            return "Error: No items provided. Please include at least one item.".to_string();
        }

        // Validate each item has required fields
        for (i, item) in request.items.iter().enumerate() {
            if !item.is_object() {
                return format!("Error: Item {i} is not a valid object.");
            }
            let obj = item.as_object().unwrap();
            if !obj.contains_key("name") {
                return format!("Error: Item {i} is missing required 'name' field.");
            }
            if !obj.contains_key("description") {
                return format!("Error: Item {i} is missing required 'description' field.");
            }
            // Validate field types
            if !obj["name"].is_string() {
                return format!("Error: Item {i} 'name' field must be a string.");
            }
            if !obj["description"].is_string() {
                return format!("Error: Item {i} 'description' field must be a string.");
            }
            // Validate optional fields if present
            if let Some(cats) = obj.get("categories") {
                if !cats.is_array() {
                    return format!("Error: Item {i} 'categories' field must be an array.");
                }
                for cat in cats.as_array().unwrap() {
                    if !cat.is_string() {
                        return format!("Error: Item {i} categories must all be strings.");
                    }
                }
            }
            if let Some(notes) = obj.get("notes") {
                if !notes.is_string() {
                    return format!("Error: Item {i} 'notes' field must be a string.");
                }
            }
        }

        // Create subdirectory path
        let subdir_path = self.directory.join(&request.name);

        // Create subdirectory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&subdir_path) {
            return format!("Error creating subdirectory '{}': {}", request.name, e);
        }

        // Generate timestamp filename
        let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S");
        let filename = format!("{timestamp}.json");
        let file_path = subdir_path.join(&filename);

        // Write JSON file
        // serde_json::Value always serializes successfully
        let json_content = serde_json::to_string_pretty(&request.items)
            .expect("serde_json::Value should always serialize");

        match fs::write(&file_path, json_content) {
            Ok(_) => {
                let items_count = request.items.len();
                let file_path_str = file_path.display();
                tracing::info!("Successfully wrote {items_count} items to {file_path_str}");
                format!(
                    "Successfully submitted {} items to '{}/{}'. Items saved to: {}",
                    request.items.len(),
                    request.name,
                    filename,
                    file_path.display()
                )
            }
            Err(e) => {
                tracing::error!("Failed to write file: {}", e);
                format!("Error writing file: {e}")
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Surface Saver helps you catalog and search items in your physical space inventories. \
                 It manages JSON files that catalog items on horizontal surfaces like desks, counters, and shelves. \
                 Each item has a name, description, and optional categories and notes. \
                 \n\nThe server provides two tools:\n\
                 1) 'search' - Find items by keywords across multiple fields. \
                    When searching, you can use multiple keywords (all must match) and optionally target specific fields. \
                    For example: search for 'red notebook' to find items matching both words, \
                    or set name=true to search only in item names.\n\
                 2) 'submit' - Add new items to the inventory by creating timestamped JSON files. \
                    Items are validated and saved with timestamps like 2025-07-29-08-30-22.json.\n\n\
                 The tools operate on the directory specified when the MCP server was started."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new(PathBuf::from("/test/path"));
        assert_eq!(*server.directory, PathBuf::from("/test/path"));
    }

    #[tokio::test]
    async fn test_search_tool() {
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

        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Test finding an item
        let request = SearchRequest {
            keywords: vec!["notebook".to_string()],
            name: None,
            description: None,
            categories: None,
            notes: None,
            all: Some(true),
        };

        let result = server.search(request).await;
        assert!(result.contains("Found 1 items:"));
        assert!(result.contains("Blue Notebook"));
        assert!(result.contains("A blue spiral notebook"));

        // Test not finding an item
        let request = SearchRequest {
            keywords: vec!["pencil".to_string()],
            name: None,
            description: None,
            categories: None,
            notes: None,
            all: Some(true),
        };

        let result = server.search(request).await;
        assert_eq!(result, "No items found matching all keywords.");
    }

    #[tokio::test]
    async fn test_search_with_specific_fields() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("items.json");

        let items = vec![serde_json::json!({
            "name": "Red Pen",
            "description": "A writing instrument",
            "categories": ["red", "stationery"],
        })];

        fs::write(&json_path, serde_json::to_string(&items).unwrap()).unwrap();

        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Search in name only - should not find
        let request = SearchRequest {
            keywords: vec!["writing".to_string()],
            name: Some(true),
            description: None,
            categories: None,
            notes: None,
            all: None,
        };

        let result = server.search(request).await;
        assert_eq!(result, "No items found matching all keywords.");

        // Search in description - should find
        let request = SearchRequest {
            keywords: vec!["writing".to_string()],
            name: None,
            description: Some(true),
            categories: None,
            notes: None,
            all: None,
        };

        let result = server.search(request).await;
        assert!(result.contains("Found 1 items:"));
        assert!(result.contains("Red Pen"));
    }

    #[tokio::test]
    async fn test_search_error_handling() {
        let server = McpServer::new(PathBuf::from("/nonexistent/directory"));

        let request = SearchRequest {
            keywords: vec!["test".to_string()],
            name: None,
            description: None,
            categories: None,
            notes: None,
            all: Some(true),
        };

        let result = server.search(request).await;
        assert!(result.contains("Error searching:"));
    }

    #[test]
    fn test_server_info() {
        let server = McpServer::new(PathBuf::from("/test/path"));
        let info = server.get_info();

        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("Surface Saver helps you catalog"));
        assert!(instructions.contains("search"));
        assert!(instructions.contains("submit"));

        // Check capabilities
        let _capabilities = info.capabilities;
        // The specific structure depends on the rmcp version
    }

    #[tokio::test]
    async fn test_submit_tool_success() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        let items = vec![
            serde_json::json!({
                "name": "Test Item 1",
                "description": "A test item",
                "categories": ["test", "demo"],
                "notes": "Test notes"
            }),
            serde_json::json!({
                "name": "Test Item 2",
                "description": "Another test item"
            }),
        ];

        let request = SubmitRequest {
            name: "test-category".to_string(),
            items,
        };

        let result = server.submit(request).await;
        assert!(result.contains("Successfully submitted 2 items"));
        assert!(result.contains("test-category"));

        // Verify directory was created
        let subdir = temp_dir.path().join("test-category");
        assert!(subdir.exists());
        assert!(subdir.is_dir());

        // Verify a JSON file was created
        let files: Vec<_> = fs::read_dir(&subdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(files.len(), 1);

        // Verify file contents
        let file_path = files[0].path();
        let content = fs::read_to_string(&file_path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["name"], "Test Item 1");
        assert_eq!(parsed[1]["name"], "Test Item 2");
    }

    #[tokio::test]
    async fn test_submit_tool_invalid_name() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Test empty name
        let request = SubmitRequest {
            name: "".to_string(),
            items: vec![serde_json::json!({"name": "Item", "description": "Desc"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Invalid subdirectory name"));

        // Test name with forward slash
        let request = SubmitRequest {
            name: "invalid/name".to_string(),
            items: vec![serde_json::json!({"name": "Item", "description": "Desc"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Invalid subdirectory name"));

        // Test name with backslash
        let request = SubmitRequest {
            name: "invalid\\name".to_string(),
            items: vec![serde_json::json!({"name": "Item", "description": "Desc"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Invalid subdirectory name"));
    }

    #[tokio::test]
    async fn test_submit_tool_empty_items() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![],
        };

        let result = server.submit(request).await;
        assert!(result.contains("Error: No items provided"));
    }

    #[tokio::test]
    async fn test_submit_tool_invalid_items() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Test non-object item
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!("not an object")],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 is not a valid object"));

        // Test missing name field
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({"description": "Missing name"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 is missing required 'name' field"));

        // Test missing description field
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({"name": "Missing description"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 is missing required 'description' field"));

        // Test invalid name type
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({"name": 123, "description": "Desc"})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 'name' field must be a string"));

        // Test invalid description type
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({"name": "Name", "description": true})],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 'description' field must be a string"));

        // Test invalid categories type
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({
                "name": "Name",
                "description": "Desc",
                "categories": "not an array"
            })],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 'categories' field must be an array"));

        // Test invalid category item type
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({
                "name": "Name",
                "description": "Desc",
                "categories": [123, "valid"]
            })],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 categories must all be strings"));

        // Test invalid notes type
        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({
                "name": "Name",
                "description": "Desc",
                "notes": 456
            })],
        };
        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 0 'notes' field must be a string"));
    }

    #[tokio::test]
    async fn test_submit_tool_multiple_items_with_error() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![
                serde_json::json!({"name": "Valid Item", "description": "Valid desc"}),
                serde_json::json!({"name": "Missing description"}),
            ],
        };

        let result = server.submit(request).await;
        assert!(result.contains("Error: Item 1 is missing required 'description' field"));
    }

    #[tokio::test]
    async fn test_submit_tool_creates_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        let request = SubmitRequest {
            name: "new-subdir".to_string(),
            items: vec![serde_json::json!({
                "name": "Test Item",
                "description": "Test description"
            })],
        };

        // Verify subdirectory doesn't exist yet
        let subdir_path = temp_dir.path().join("new-subdir");
        assert!(!subdir_path.exists());

        let result = server.submit(request).await;
        assert!(result.contains("Successfully submitted 1 items"));

        // Verify subdirectory was created
        assert!(subdir_path.exists());
        assert!(subdir_path.is_dir());
    }

    #[tokio::test]
    async fn test_submit_tool_existing_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Create existing subdirectory with a file
        let subdir_path = temp_dir.path().join("existing");
        fs::create_dir(&subdir_path).unwrap();
        fs::write(subdir_path.join("existing.json"), "[]").unwrap();

        let request = SubmitRequest {
            name: "existing".to_string(),
            items: vec![serde_json::json!({
                "name": "New Item",
                "description": "New description"
            })],
        };

        let result = server.submit(request).await;
        assert!(result.contains("Successfully submitted 1 items"));

        // Verify both files exist
        let files: Vec<_> = fs::read_dir(&subdir_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_submit_tool_directory_creation_error() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();

        // Create a read-only directory to prevent creating subdirectories
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&readonly_dir, perms).unwrap();

        let server = McpServer::new(readonly_dir.clone());

        let request = SubmitRequest {
            name: "subdir".to_string(),
            items: vec![serde_json::json!({
                "name": "Test",
                "description": "Test"
            })],
        };

        let result = server.submit(request).await;
        assert!(result.contains("Error creating subdirectory"));

        // Clean up permissions
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }

    #[tokio::test]
    async fn test_submit_tool_write_error() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(temp_dir.path().to_path_buf());

        // Create a subdirectory
        let subdir = temp_dir.path().join("test");
        fs::create_dir(&subdir).unwrap();

        // Make the subdirectory read-only after creation
        let mut perms = fs::metadata(&subdir).unwrap().permissions();
        perms.set_mode(0o555); // Read and execute only
        fs::set_permissions(&subdir, perms).unwrap();

        let request = SubmitRequest {
            name: "test".to_string(),
            items: vec![serde_json::json!({
                "name": "Test",
                "description": "Test"
            })],
        };

        let result = server.submit(request).await;
        assert!(result.contains("Error writing file"));

        // Clean up permissions
        let mut perms = fs::metadata(&subdir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&subdir, perms).unwrap();
    }
}
