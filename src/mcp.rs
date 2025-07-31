use std::path::PathBuf;
use std::sync::Arc;

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
        // TODO: Implement actual MCP server when rmcp API stabilizes
        // For now, just print a message
        eprintln!(
            "MCP server would start with directory: {:?}",
            self.directory
        );
        eprintln!("This feature is not yet implemented.");
        Err("MCP server not yet implemented".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_new() {
        let server = McpServer::new(PathBuf::from("/test/path"));
        assert_eq!(*server.directory, PathBuf::from("/test/path"));
    }

    #[tokio::test]
    async fn test_mcp_server_run_not_implemented() {
        let server = McpServer::new(PathBuf::from("/test/path"));
        let result = server.run().await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }
}
