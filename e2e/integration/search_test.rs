/// Integration tests for search functionality
///
/// Tests semantic search with various queries, filters, and parameters

#[cfg(test)]
mod search_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
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

        fn cmd(&self) -> Command {
            let mut cmd = Command::cargo_bin("dna").unwrap();
            cmd.current_dir(self.root());
            cmd
        }

        fn init(&self) {
            self.cmd().arg("init").assert().success();
        }

        fn add_artifact(&self, artifact_type: &str, content: &str, metadata: &[(&str, &str)]) {
            let mut args = vec!["add", artifact_type, content];

            for (key, value) in metadata {
                args.push("--meta");
                args.push(&format!("{}={}", key, value));
            }

            self.cmd().args(&args).assert().success();
        }
    }

    #[test]
    fn test_search_basic() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication via email and password", &[]);
        ctx.add_artifact("intent", "Payment processing workflow", &[]);

        ctx.cmd()
            .args(&["search", "authentication"])
            .assert()
            .success()
            .stdout(predicate::str::contains("authentication"));
    }

    #[test]
    fn test_search_semantic_similarity() {
        let ctx = TestContext::new();
        ctx.init();

        // Add semantically related content
        ctx.add_artifact("intent", "Users log in with email and password", &[]);
        ctx.add_artifact("intent", "System processes credit card payments", &[]);
        ctx.add_artifact("intent", "Authentication via OAuth providers", &[]);

        // Search for "login" should find authentication-related artifacts
        let output = ctx.cmd().args(&["search", "login"]).output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();

        assert!(stdout.contains("log in") || stdout.contains("Authentication"));
    }

    #[test]
    fn test_search_with_type_filter() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication system", &[]);
        ctx.add_artifact("invariant", "Users must verify email", &[]);
        ctx.add_artifact("contract", "POST /auth/login API endpoint", &[]);

        ctx.cmd()
            .args(&["search", "authentication", "--kind", "intent"])
            .assert()
            .success()
            .stdout(predicate::str::contains("intent"))
            .stdout(predicate::str::contains("invariant").not());
    }

    #[test]
    fn test_search_with_metadata_filter() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Auth feature", &[("domain", "auth"), ("priority", "high")]);
        ctx.add_artifact("intent", "Payment feature", &[("domain", "payment"), ("priority", "low")]);

        ctx.cmd()
            .args(&["search", "feature", "--filter", "domain=auth"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Auth feature"))
            .stdout(predicate::str::contains("Payment feature").not());
    }

    #[test]
    fn test_search_with_multiple_filters() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact(
            "intent",
            "Critical auth feature",
            &[("domain", "auth"), ("priority", "critical")],
        );
        ctx.add_artifact(
            "intent",
            "Important auth feature",
            &[("domain", "auth"), ("priority", "high")],
        );
        ctx.add_artifact(
            "intent",
            "Critical payment feature",
            &[("domain", "payment"), ("priority", "critical")],
        );

        ctx.cmd()
            .args(&[
                "search",
                "feature",
                "--filter",
                "domain=auth",
                "--filter",
                "priority=critical",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Critical auth feature"))
            .stdout(predicate::str::contains("Important auth").not())
            .stdout(predicate::str::contains("payment").not());
    }

    #[test]
    fn test_search_with_limit() {
        let ctx = TestContext::new();
        ctx.init();

        // Add 10 similar artifacts
        for i in 0..10 {
            ctx.add_artifact("intent", &format!("Feature number {}", i), &[]);
        }

        let output = ctx.cmd().args(&["search", "feature", "--limit", "3"]).output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();
        let count = stdout.lines().filter(|line| line.contains("Feature number")).count();

        assert!(count <= 3, "Should return at most 3 results, got {}", count);
    }

    #[test]
    fn test_search_no_results() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication", &[]);

        ctx.cmd()
            .args(&["search", "nonexistent_query_xyz"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No results found").or(predicate::str::contains("0 results")));
    }

    #[test]
    fn test_search_empty_database() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.cmd()
            .args(&["search", "anything"])
            .assert()
            .success()
            .stdout(predicate::str::contains("No results found").or(predicate::str::contains("0 results")));
    }

    #[test]
    fn test_search_relevance_ranking() {
        let ctx = TestContext::new();
        ctx.init();

        // Add artifacts with varying relevance to query
        ctx.add_artifact("intent", "User authentication and authorization system", &[]);
        ctx.add_artifact("intent", "Authentication via email", &[]);
        ctx.add_artifact("intent", "Payment processing", &[]);

        let output = ctx.cmd().args(&["search", "authentication"]).output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        // More relevant results should appear first
        let auth_pos = stdout.find("authentication").unwrap_or(usize::MAX);
        let payment_pos = stdout.find("Payment").unwrap_or(usize::MAX);

        assert!(auth_pos < payment_pos, "More relevant results should rank higher");
    }

    #[test]
    fn test_search_with_score_display() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication system", &[]);

        ctx.cmd()
            .args(&["search", "authentication", "--show-scores"])
            .assert()
            .success()
            .stdout(predicate::str::is_match(r"Score: \d+\.\d+").unwrap().or(predicate::str::contains("similarity")));
    }

    #[test]
    fn test_search_case_insensitive() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User Authentication System", &[]);

        ctx.cmd()
            .args(&["search", "authentication"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Authentication"));

        ctx.cmd()
            .args(&["search", "AUTHENTICATION"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Authentication"));
    }

    #[test]
    fn test_search_partial_words() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication and authorization", &[]);

        ctx.cmd()
            .args(&["search", "auth"])
            .assert()
            .success()
            .stdout(predicate::str::contains("authentication"));
    }

    #[test]
    fn test_search_multi_word_query() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication via email and password", &[]);
        ctx.add_artifact("intent", "OAuth authentication provider", &[]);

        ctx.cmd()
            .args(&["search", "email password authentication"])
            .assert()
            .success()
            .stdout(predicate::str::contains("email and password"));
    }

    #[test]
    fn test_search_unicode_query() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Support for 多语言 multilingual content", &[]);

        ctx.cmd()
            .args(&["search", "多语言"])
            .assert()
            .success()
            .stdout(predicate::str::contains("多语言"));
    }

    #[test]
    fn test_search_with_special_characters() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("contract", "POST /api/v1/users/{id}", &[]);

        ctx.cmd()
            .args(&["search", "/api/v1/users"])
            .assert()
            .success()
            .stdout(predicate::str::contains("/api/v1/users"));
    }

    #[test]
    fn test_search_across_types() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Payment processing system", &[]);
        ctx.add_artifact("invariant", "Payment must be validated", &[]);
        ctx.add_artifact("contract", "POST /api/payments", &[]);

        let output = ctx.cmd().args(&["search", "payment"]).output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should find all types
        assert!(
            stdout.contains("processing") || stdout.contains("validated") || stdout.contains("payments"),
            "Search should find artifacts across all types"
        );
    }

    #[test]
    fn test_search_recent_artifacts() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Old feature", &[]);
        std::thread::sleep(std::time::Duration::from_millis(100));
        ctx.add_artifact("intent", "New feature", &[]);

        // Search with recent bias (if supported)
        ctx.cmd()
            .args(&["search", "feature", "--recent"])
            .assert()
            .success();
    }

    #[test]
    fn test_search_with_json_output() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Test content", &[]);

        let output = ctx.cmd().args(&["search", "test", "--format", "json"]).output().unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should be valid JSON
        let json: serde_json::Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(json.is_array() || json.is_object());
    }

    #[test]
    fn test_search_performance_large_dataset() {
        let ctx = TestContext::new();
        ctx.init();

        // Add 100 artifacts
        for i in 0..100 {
            ctx.add_artifact("intent", &format!("Feature description number {}", i), &[]);
        }

        let start = std::time::Instant::now();
        ctx.cmd().args(&["search", "feature"]).assert().success();
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 5,
            "Search should complete in <5s, took {:?}",
            duration
        );
    }

    #[test]
    fn test_search_with_threshold() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "User authentication", &[]);
        ctx.add_artifact("intent", "Something completely different", &[]);

        // Only return results above similarity threshold
        ctx.cmd()
            .args(&["search", "authentication", "--threshold", "0.5"])
            .assert()
            .success()
            .stdout(predicate::str::contains("authentication"))
            .stdout(predicate::str::contains("completely different").not());
    }

    #[test]
    fn test_changes_command() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Initial content", &[]);

        std::thread::sleep(std::time::Duration::from_millis(100));
        let since = chrono::Utc::now().to_rfc3339();

        std::thread::sleep(std::time::Duration::from_millis(100));
        ctx.add_artifact("intent", "New content", &[]);

        ctx.cmd()
            .args(&["changes", "--since", &since])
            .assert()
            .success()
            .stdout(predicate::str::contains("New content"))
            .stdout(predicate::str::contains("Initial content").not());
    }

    #[test]
    fn test_search_error_without_init() {
        let ctx = TestContext::new();

        ctx.cmd()
            .args(&["search", "anything"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("Run 'dna init'")));
    }

    #[test]
    fn test_search_with_empty_query() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.cmd()
            .args(&["search", ""])
            .assert()
            .failure()
            .stderr(predicate::str::contains("empty").or(predicate::str::contains("required")));
    }
}
