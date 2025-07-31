#!/bin/bash

# Test script to demonstrate MCP server functionality
# This script sends a basic MCP initialize request to verify the server starts

echo "Testing MCP server with initialize request..."

# Create a test directory with sample data
TEST_DIR=$(mktemp -d)
mkdir -p "$TEST_DIR/category1"

cat > "$TEST_DIR/category1/items.json" << EOF
[
  {
    "name": "Test Item",
    "description": "A test item for MCP server demo"
  }
]
EOF

# Send initialize request to MCP server
# The server expects JSON-RPC messages over stdio
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocol_version":"1.0.0","client_info":{"name":"test-client","version":"1.0"}},"id":1}' | cargo run -- mcp "$TEST_DIR" 2>/dev/null | head -1

# Clean up
rm -rf "$TEST_DIR"

echo "MCP server test complete."