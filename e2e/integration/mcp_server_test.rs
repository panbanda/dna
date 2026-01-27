/// Integration tests for MCP server tools
///
/// Tests MCP server functionality including tool discovery and invocation

#[cfg(test)]
mod mcp_server_tests {
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    struct McpServer {
        process: Child,
        temp_dir: TempDir,
    }

    impl McpServer {
        fn start() -> Self {
            let temp_dir = TempDir::new().unwrap();

            // Initialize DNA in temp directory
            Command::new(env!("CARGO_BIN_EXE_dna"))
                .current_dir(temp_dir.path())
                .arg("init")
                .output()
                .expect("Failed to initialize DNA");

            // Start MCP server
            let mut process = Command::new(env!("CARGO_BIN_EXE_dna"))
                .current_dir(temp_dir.path())
                .arg("mcp")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start MCP server");

            // Wait a bit for server to start
            thread::sleep(Duration::from_millis(500));

            Self { process, temp_dir }
        }

        fn send_request(&mut self, method: &str, params: Value) -> Value {
            let request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            });

            let stdin = self.process.stdin.as_mut().unwrap();
            writeln!(stdin, "{}", request.to_string()).unwrap();
            stdin.flush().unwrap();

            // Read response
            let stdout = self.process.stdout.as_mut().unwrap();
            let reader = BufReader::new(stdout);

            for line in reader.lines().take(1) {
                if let Ok(line) = line {
                    return serde_json::from_str(&line).unwrap_or(json!({}));
                }
            }

            json!({})
        }

        fn stop(&mut self) {
            let _ = self.process.kill();
        }
    }

    impl Drop for McpServer {
        fn drop(&mut self) {
            self.stop();
        }
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_server_starts() {
        let server = McpServer::start();
        assert!(server.process.id() > 0, "Server should be running");
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_list_tools() {
        let mut server = McpServer::start();

        let response = server.send_request("tools/list", json!({}));

        assert!(response["result"].is_array());

        let tools = response["result"].as_array().unwrap();
        let tool_names: Vec<_> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        // Should include all DNA tools
        assert!(tool_names.contains(&"dna_search"));
        assert!(tool_names.contains(&"dna_get"));
        assert!(tool_names.contains(&"dna_list"));
        assert!(tool_names.contains(&"dna_changes"));
        assert!(tool_names.contains(&"dna_add"));
        assert!(tool_names.contains(&"dna_update"));
        assert!(tool_names.contains(&"dna_remove"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_tool_schemas() {
        let mut server = McpServer::start();

        let response = server.send_request("tools/list", json!({}));
        let tools = response["result"].as_array().unwrap();

        for tool in tools {
            // Each tool should have required fields
            assert!(tool["name"].is_string());
            assert!(tool["description"].is_string());
            assert!(tool["inputSchema"].is_object());
        }
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_search_tool() {
        let mut server = McpServer::start();

        // Add an artifact first via DNA CLI
        Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "User authentication system"])
            .output()
            .unwrap();

        // Search via MCP
        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_search",
                "arguments": {
                    "query": "authentication",
                    "limit": 10
                }
            }),
        );

        assert!(response["result"].is_object());
        let result = &response["result"];
        assert!(result["content"].is_array());

        let content = result["content"].as_array().unwrap();
        assert!(!content.is_empty());

        let first_result = &content[0];
        assert!(first_result["text"].as_str().unwrap().contains("authentication"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_get_tool() {
        let mut server = McpServer::start();

        // Add artifact and extract ID
        let output = Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "Test content"])
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        let id = extract_id(&stdout);

        // Get via MCP
        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_get",
                "arguments": {
                    "id": id
                }
            }),
        );

        assert!(response["result"].is_object());
        assert!(response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Test content"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_add_tool() {
        let mut server = McpServer::start();

        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_add",
                "arguments": {
                    "type": "intent",
                    "content": "New artifact via MCP",
                    "name": "mcp-artifact",
                    "metadata": {
                        "source": "mcp-test"
                    }
                }
            }),
        );

        assert!(response["result"].is_object());

        let result_text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(result_text.contains("Added") || result_text.contains("Created"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_update_tool() {
        let mut server = McpServer::start();

        // Add artifact
        let output = Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "Original content"])
            .output()
            .unwrap();

        let id = extract_id(&String::from_utf8(output.stdout).unwrap());

        // Update via MCP
        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_update",
                "arguments": {
                    "id": id,
                    "content": "Updated content",
                }
            }),
        );

        assert!(response["result"].is_object());

        // Verify update
        let get_response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_get",
                "arguments": { "id": id }
            }),
        );

        assert!(get_response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Updated content"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_remove_tool() {
        let mut server = McpServer::start();

        // Add artifact
        let output = Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "To be removed"])
            .output()
            .unwrap();

        let id = extract_id(&String::from_utf8(output.stdout).unwrap());

        // Remove via MCP
        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_remove",
                "arguments": { "id": id }
            }),
        );

        assert!(response["result"].is_object());

        // Verify removal
        let get_response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_get",
                "arguments": { "id": id }
            }),
        );

        assert!(get_response["error"].is_object());
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_list_tool() {
        let mut server = McpServer::start();

        // Add multiple artifacts
        for i in 0..5 {
            Command::new(env!("CARGO_BIN_EXE_dna"))
                .current_dir(server.temp_dir.path())
                .args(&["intent", "add", &format!("Content {}", i)])
                .output()
                .unwrap();
        }

        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_list",
                "arguments": {
                    "type": "intent"
                }
            }),
        );

        assert!(response["result"].is_object());
        let content = &response["result"]["content"][0]["text"].as_str().unwrap();

        assert!(content.contains("Content 0"));
        assert!(content.contains("Content 4"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_changes_tool() {
        let mut server = McpServer::start();

        // Add artifact
        Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "Old content"])
            .output()
            .unwrap();

        thread::sleep(Duration::from_millis(100));
        let since = chrono::Utc::now().to_rfc3339();
        thread::sleep(Duration::from_millis(100));

        // Add new artifact
        Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(server.temp_dir.path())
            .args(&["intent", "add", "New content"])
            .output()
            .unwrap();

        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_changes",
                "arguments": {
                    "since": since
                }
            }),
        );

        assert!(response["result"].is_object());
        let content = response["result"]["content"][0]["text"].as_str().unwrap();

        assert!(content.contains("New content"));
        assert!(!content.contains("Old content"));
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_read_only_mode() {
        let temp_dir = TempDir::new().unwrap();

        Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(temp_dir.path())
            .arg("init")
            .output()
            .unwrap();

        // Start server in read-only mode
        let mut process = Command::new(env!("CARGO_BIN_EXE_dna"))
            .current_dir(temp_dir.path())
            .args(&["mcp", "--exclude", "add,update,remove"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(500));

        // Request tools list
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        let stdin = process.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", request.to_string()).unwrap();
        stdin.flush().unwrap();

        // Verify write tools are excluded
        let stdout = process.stdout.as_mut().unwrap();
        let reader = BufReader::new(stdout);

        for line in reader.lines().take(1) {
            if let Ok(line) = line {
                let response: Value = serde_json::from_str(&line).unwrap_or(json!({}));
                let tools = response["result"].as_array().unwrap();
                let tool_names: Vec<_> = tools
                    .iter()
                    .filter_map(|t| t["name"].as_str())
                    .collect();

                assert!(tool_names.contains(&"dna_search"));
                assert!(tool_names.contains(&"dna_get"));
                assert!(!tool_names.contains(&"dna_add"));
                assert!(!tool_names.contains(&"dna_update"));
                assert!(!tool_names.contains(&"dna_remove"));
            }
        }

        let _ = process.kill();
    }

    #[test]
    #[ignore] // Requires full MCP implementation
    fn test_mcp_error_handling() {
        let mut server = McpServer::start();

        // Call tool with invalid parameters
        let response = server.send_request(
            "tools/call",
            json!({
                "name": "dna_get",
                "arguments": {
                    "id": "invalid_id_format"
                }
            }),
        );

        assert!(response["error"].is_object());
        assert!(response["error"]["message"].is_string());
    }

    fn extract_id(output: &str) -> String {
        let re = regex::Regex::new(r"ID: ([a-z2-9]{10})").unwrap();
        re.captures(output)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .expect("Failed to extract ID")
    }
}
