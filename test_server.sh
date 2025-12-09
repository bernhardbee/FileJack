#!/bin/bash

# FileJack MCP Server Test Script
# This script demonstrates how to interact with FileJack server

echo "=== FileJack MCP Server Test ==="
echo ""

# Build the project
echo "Building FileJack..."
cargo build --release --quiet
echo "✓ Build complete"
echo ""

# Test 1: Initialize
echo "Test 1: Initialize the server"
echo '{"jsonrpc":"2.0","method":"initialize","id":1}' | \
  timeout 2 ./target/release/filejack 2>/dev/null | \
  python3 -m json.tool
echo ""

# Test 2: List tools
echo "Test 2: List available tools"
echo '{"jsonrpc":"2.0","method":"tools/list","id":2}' | \
  timeout 2 ./target/release/filejack 2>/dev/null | \
  python3 -m json.tool
echo ""

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
echo "Using test directory: $TEST_DIR"
echo ""

# Test 3: Write a file
echo "Test 3: Write a file"
FILE_PATH="$TEST_DIR/test.txt"
echo "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"write_file\",\"arguments\":{\"path\":\"$FILE_PATH\",\"content\":\"Hello from FileJack!\"}}, \"id\":3}" | \
  FILEJACK_BASE_PATH="$TEST_DIR" timeout 2 ./target/release/filejack 2>/dev/null | \
  python3 -m json.tool
echo ""

# Verify the file was created
if [ -f "$FILE_PATH" ]; then
  echo "✓ File created successfully"
  echo "  Content: $(cat $FILE_PATH)"
else
  echo "✗ File was not created"
fi
echo ""

# Test 4: Read the file
echo "Test 4: Read the file"
echo "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"read_file\",\"arguments\":{\"path\":\"$FILE_PATH\"}}, \"id\":4}" | \
  FILEJACK_BASE_PATH="$TEST_DIR" timeout 2 ./target/release/filejack 2>/dev/null | \
  python3 -m json.tool
echo ""

# Test 5: Error handling - read non-existent file
echo "Test 5: Error handling - read non-existent file"
NONEXISTENT="$TEST_DIR/nonexistent.txt"
echo "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"read_file\",\"arguments\":{\"path\":\"$NONEXISTENT\"}}, \"id\":5}" | \
  FILEJACK_BASE_PATH="$TEST_DIR" timeout 2 ./target/release/filejack 2>/dev/null | \
  python3 -m json.tool
echo ""

# Clean up
rm -rf "$TEST_DIR"
echo "✓ Test directory cleaned up"
echo ""
echo "=== All tests completed ==="
