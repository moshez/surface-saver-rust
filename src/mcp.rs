use crate::search::{SearchOptions, search_directory};
use anyhow::Result;
use rmcp::{
    ServerHandler, ServiceExt,
    model::{ServerCapabilities, ServerInfo},
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{stdin, stdout};
use tracing_subscriber::EnvFilter;

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

#[derive(Debug, Clone)]
pub struct McpServer {
    directory: Arc<PathBuf>,
}

impl McpServer {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Arc::new(directory),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Set up tracing to stderr so it doesn't interfere with stdio transport
        // Only initialize if not already initialized (e.g., in tests)
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
            )
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .try_init();

        tracing::info!("Starting Surface Saver MCP server");
        tracing::info!("Directory: {:?}", self.directory);

        // Create stdio transport
        let transport = (stdin(), stdout());

        // Serve the MCP server
        let service = self.serve(transport).await.inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

        // Wait for the service to complete
        match service.waiting().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Service error: {e:?}").into()),
        }
    }
}

#[tool(tool_box)]
impl McpServer {
    #[tool(description = "Search for items in the inventory by keywords")]
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
}

#[tool(tool_box)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Surface Saver helps search through inventory JSON files. \
                 Use the search tool to find items by keywords across multiple fields."
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
        assert!(
            info.instructions
                .unwrap()
                .contains("Surface Saver helps search")
        );

        // Check capabilities
        let _capabilities = info.capabilities;
        // The specific structure depends on the rmcp version
    }
}
