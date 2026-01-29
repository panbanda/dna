/// Integration tests for CLI commands
///
/// Tests the complete CLI workflow: init, add, get, update, remove, list

#[cfg(test)]
mod cli_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct TestContext {
        temp_dir: TempDir,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                temp_dir: TempDir::new().unwrap(),
            }
        }

        fn root(&self) -> PathBuf {
            self.temp_dir.path().to_path_buf()
        }

        fn dna_dir(&self) -> PathBuf {
            self.root().join(".dna")
        }

        fn cmd(&self) -> Command {
            let mut cmd = Command::cargo_bin("dna").unwrap();
            cmd.current_dir(self.root());
            cmd
        }
    }

    #[test]
    fn test_init_creates_directory() {
        let ctx = TestContext::new();

        ctx.cmd()
            .arg("init")
            .assert()
            .success()
            .stdout(predicate::str::contains("Initialized DNA"));

        assert!(ctx.dna_dir().exists());
        assert!(ctx.dna_dir().join("config.toml").exists());
        assert!(ctx.dna_dir().join("db").exists());
    }

    #[test]
    fn test_init_with_custom_model() {
        let ctx = TestContext::new();

        ctx.cmd()
            .args(&["init", "--model", "openai:text-embedding-3-small"])
            .assert()
            .success();

        let config = fs::read_to_string(ctx.dna_dir().join("config.toml")).unwrap();
        assert!(config.contains("provider = \"openai\""));
        assert!(config.contains("name = \"text-embedding-3-small\""));
    }

    #[test]
    fn test_init_already_initialized() {
        let ctx = TestContext::new();

        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .arg("init")
            .assert()
            .failure()
            .stderr(predicate::str::contains("already initialized"));
    }

    #[test]
    fn test_intent_add() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "intent", "The system authenticates users via email"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Added artifact"))
            .stdout(predicate::str::is_match(r"ID: [a-z2-9]{10}").unwrap());
    }

    #[test]
    fn test_intent_add_with_name() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&[
                "intent",
                "add",
                "The system authenticates users",
                "--name",
                "user-auth",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("user-auth"));
    }

    #[test]
    fn test_intent_add_with_metadata() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&[
                "intent",
                "add",
                "Content",
                "--meta",
                "domain=auth",
                "--meta",
                "priority=high",
            ])
            .assert()
            .success();
    }

    #[test]
    fn test_all_artifact_types() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let types = vec![
            "intent",
            "invariant",
            "contract",
            "algorithm",
            "evaluation",
            "pace",
            "monitor",
        ];

        for artifact_type in types {
            ctx.cmd()
                .args(&["add", artifact_type, &format!("Test {} content", artifact_type)])
                .assert()
                .success()
                .stdout(predicate::str::contains(format!("Added artifact")));
        }
    }

    #[test]
    fn test_get_artifact() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let output = ctx
            .cmd()
            .args(&["add", "intent", "Test content"])
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        let id = extract_id(&stdout);

        ctx.cmd()
            .args(&["get", &id])
            .assert()
            .success()
            .stdout(predicate::str::contains("Test content"))
            .stdout(predicate::str::contains(&id));
    }

    #[test]
    fn test_get_nonexistent() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["get", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_update_artifact_content() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let output = ctx
            .cmd()
            .args(&["add", "intent", "Original content"])
            .output()
            .unwrap();

        let id = extract_id(&String::from_utf8(output.stdout).unwrap());

        ctx.cmd()
            .args(&["update", &id, "--content", "Updated content"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Updated"));

        ctx.cmd()
            .args(&["get", &id])
            .assert()
            .stdout(predicate::str::contains("Updated content"));
    }

    #[test]
    fn test_update_artifact_metadata() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let output = ctx.cmd().args(&["add", "intent", "Content"]).output().unwrap();
        let id = extract_id(&String::from_utf8(output.stdout).unwrap());

        ctx.cmd()
            .args(&["update", &id, "--meta", "status=reviewed"])
            .assert()
            .success();
    }

    #[test]
    fn test_remove_artifact() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let output = ctx.cmd().args(&["add", "intent", "Content"]).output().unwrap();
        let id = extract_id(&String::from_utf8(output.stdout).unwrap());

        ctx.cmd()
            .args(&["remove", &id])
            .assert()
            .success()
            .stdout(predicate::str::contains("Removed"));

        ctx.cmd()
            .args(&["get", &id])
            .assert()
            .failure();
    }

    #[test]
    fn test_remove_nonexistent() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["remove", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not found"));
    }

    #[test]
    fn test_list_artifacts() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        // Add multiple artifacts
        for i in 0..5 {
            ctx.cmd()
                .args(&["add", "intent", &format!("Content {}", i)])
                .assert()
                .success();
        }

        ctx.cmd()
            .args(&["list", "--kind", "intent"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Content 0"))
            .stdout(predicate::str::contains("Content 4"));
    }

    #[test]
    fn test_list_with_filter() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "intent", "Content 1", "--meta", "priority=high"])
            .assert()
            .success();

        ctx.cmd()
            .args(&["add", "intent", "Content 2", "--meta", "priority=low"])
            .assert()
            .success();

        ctx.cmd()
            .args(&["list", "--kind", "intent", "--filter", "priority=high"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Content 1"))
            .stdout(predicate::str::contains("Content 2").not());
    }

    #[test]
    fn test_list_with_limit() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        for i in 0..10 {
            ctx.cmd()
                .args(&["add", "intent", &format!("Content {}", i)])
                .assert()
                .success();
        }

        let output = ctx.cmd().args(&["list", "--kind", "intent", "--limit", "3"]).output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        let line_count = stdout.lines().filter(|line| line.contains("Content")).count();

        assert_eq!(line_count, 3);
    }

    #[test]
    fn test_list_empty() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["list", "--kind", "intent"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No artifacts found"));
    }

    #[test]
    fn test_cross_type_operations() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        // Add artifacts of different types
        ctx.cmd().args(&["add", "intent", "Intent content"]).assert().success();
        ctx.cmd().args(&["add", "invariant", "Invariant content"]).assert().success();

        // List all artifacts
        ctx.cmd()
            .args(&["list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Intent content"))
            .stdout(predicate::str::contains("Invariant content"));
    }

    #[test]
    fn test_config_get() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["config", "get", "model.provider"])
            .assert()
            .success()
            .stdout(predicate::str::contains("local"));
    }

    #[test]
    fn test_config_set() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["config", "set", "model.provider", "openai"])
            .assert()
            .success();

        ctx.cmd()
            .args(&["config", "get", "model.provider"])
            .assert()
            .stdout(predicate::str::contains("openai"));
    }

    #[test]
    fn test_config_model() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["config", "model", "openai:text-embedding-3-small"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Model updated"));
    }

    #[test]
    fn test_reindex() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd().args(&["add", "intent", "Content"]).assert().success();

        ctx.cmd()
            .args(&["reindex"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Reindexing"));
    }

    #[test]
    fn test_reindex_force() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd().args(&["add", "intent", "Content"]).assert().success();

        ctx.cmd()
            .args(&["reindex", "--force"])
            .assert()
            .success();
    }

    #[test]
    fn test_error_without_init() {
        let ctx = TestContext::new();

        ctx.cmd()
            .args(&["add", "intent", "Content"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("Run 'dna init'")));
    }

    #[test]
    fn test_help_command() {
        Command::cargo_bin("dna")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("DNA CLI"))
            .stdout(predicate::str::contains("USAGE"));
    }

    #[test]
    fn test_version_command() {
        Command::cargo_bin("dna")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
    }

    #[test]
    fn test_artifact_format_json() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "intent", r#"{"key": "value"}"#, "--format", "json"])
            .assert()
            .success();
    }

    #[test]
    fn test_artifact_format_yaml() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "contract", "key: value", "--format", "yaml"])
            .assert()
            .success();
    }

    #[test]
    fn test_artifact_format_openapi() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "contract", "openapi: 3.0.0", "--format", "openapi"])
            .assert()
            .success();
    }

    #[test]
    fn test_unicode_content() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let unicode_content = "Hello 世界 🌍 مرحبا";

        ctx.cmd()
            .args(&["add", "intent", unicode_content])
            .assert()
            .success();
    }

    #[test]
    fn test_large_content() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        let large_content = "x".repeat(10_000);

        ctx.cmd()
            .args(&["add", "intent", &large_content])
            .assert()
            .success();
    }

    #[test]
    fn test_special_characters_in_name() {
        let ctx = TestContext::new();
        ctx.cmd().arg("init").assert().success();

        ctx.cmd()
            .args(&["add", "intent", "Content", "--name", "test-name_123"])
            .assert()
            .success();
    }

    // Helper functions

    fn extract_id(output: &str) -> String {
        let re = regex::Regex::new(r"ID: ([a-z2-9]{10})").unwrap();
        re.captures(output)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .expect("Failed to extract ID from output")
    }

    #[cfg(test)]
    mod serial_tests {
        use super::*;
        use serial_test::serial;

        #[test]
        #[serial]
        fn test_concurrent_add_operations() {
            let ctx = TestContext::new();
            ctx.cmd().arg("init").assert().success();

            use std::thread;

            let mut handles = vec![];
            let root = ctx.root();

            for i in 0..5 {
                let root_clone = root.clone();
                let handle = thread::spawn(move || {
                    let mut cmd = Command::cargo_bin("dna").unwrap();
                    cmd.current_dir(root_clone);
                    cmd.args(&["add", "intent", &format!("Concurrent content {}", i)])
                        .assert()
                        .success();
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            let output = ctx.cmd().args(&["list", "--kind", "intent"]).output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            let count = stdout.lines().filter(|line| line.contains("Concurrent")).count();

            assert_eq!(count, 5);
        }
    }
}
