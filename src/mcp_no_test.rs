use crate::mcp::McpServer;
use rmcp::ServiceExt;
use rmcp::service::QuitReason;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing_subscriber::EnvFilter;

impl McpServer {
    pub async fn run<R, W>(
        self,
        reader: R,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
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

        // Create transport from provided reader/writer
        let transport = (reader, writer);

        // Serve the MCP server
        let _service = self.serve(transport).await.inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

        // Wait for the service to complete
        let waiting_result = _service.waiting().await;
        handle_service_completion(waiting_result)
    }
}

// Helper function to handle service completion - extracted for testability
fn handle_service_completion(
    result: Result<QuitReason, tokio::task::JoinError>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match result {
        Ok(_) => {
            tracing::info!("MCP server exited successfully");
            Ok(())
        }
        Err(e) => Err(format!("Service error: {e:?}").into()),
    }
}
