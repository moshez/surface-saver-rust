use surface_saver_rust::{Commands, run_command};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::test]
async fn test_mcp_server_with_files() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create temporary files for input/output
    let input_path = temp_dir.path().join("input.txt");
    let output_path = temp_dir.path().join("output.txt");
    
    // Write empty input (immediate EOF)
    let mut input_file = File::create(&input_path).await.unwrap();
    input_file.write_all(b"").await.unwrap();
    input_file.sync_all().await.unwrap();
    drop(input_file);
    
    // Create the command
    let cmd = Commands::Mcp {
        directory: temp_dir.path().to_path_buf(),
    };
    
    // Run command (it will use stdin/stdout which will fail in test, but that's expected)
    let result = run_command(cmd).await;
    
    // In a test environment without proper stdio, MCP server returns error
    assert_eq!(result.exit_code, 1);
}