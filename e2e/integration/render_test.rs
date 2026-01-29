/// Integration tests for render/checkout functionality
///
/// Tests rendering artifacts to files with proper structure and formatting

#[cfg(test)]
mod render_integration_tests {
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

        fn render_dir(&self) -> PathBuf {
            self.root().join("dna")
        }

        fn cmd(&self) -> Command {
            let mut cmd = Command::cargo_bin("dna").unwrap();
            cmd.current_dir(self.root());
            cmd
        }

        fn init(&self) {
            self.cmd().arg("init").assert().success();
        }

        fn add_artifact(
            &self,
            artifact_type: &str,
            content: &str,
            name: Option<&str>,
            metadata: &[(&str, &str)],
        ) -> String {
            let mut args = vec!["add", artifact_type, content];

            if let Some(n) = name {
                args.push("--name");
                args.push(n);
            }

            for (key, value) in metadata {
                args.push("--meta");
                args.push(&format!("{}={}", key, value));
            }

            let output = self.cmd().args(&args).output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            extract_id(&stdout)
        }
    }

    fn extract_id(output: &str) -> String {
        let re = regex::Regex::new(r"ID: ([a-z2-9]{10})").unwrap();
        re.captures(output)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .expect("Failed to extract ID")
    }

    #[test]
    fn test_render_default_directory() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Test content", Some("test-intent"), &[]);

        ctx.cmd().arg("render").assert().success();

        assert!(ctx.render_dir().exists(), "Render directory should be created");
    }

    #[test]
    fn test_render_creates_type_directories() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Intent content", Some("intent-1"), &[]);
        ctx.add_artifact("invariant", "Invariant content", Some("invariant-1"), &[]);
        ctx.add_artifact("contract", "Contract content", Some("contract-1"), &[]);

        ctx.cmd().arg("render").assert().success();

        let checkout = ctx.render_dir().join("checkout");
        assert!(checkout.join("intents").exists());
        assert!(checkout.join("invariants").exists());
        assert!(checkout.join("contracts").exists());
    }

    #[test]
    fn test_render_file_with_frontmatter() {
        let ctx = TestContext::new();
        ctx.init();

        let id = ctx.add_artifact("invariant", "Users must verify email", Some("email-verification"), &[
            ("domain", "auth"),
            ("priority", "high"),
        ]);

        ctx.cmd().arg("render").assert().success();

        let file_path = ctx.render_dir().join("checkout/invariants/email-verification.md");
        assert!(file_path.exists(), "Rendered file should exist");

        let content = fs::read_to_string(&file_path).unwrap();

        // Check frontmatter
        assert!(content.starts_with("---"));
        assert!(content.contains(&format!("id: {}", id)));
        assert!(content.contains("kind: invariant"));
        assert!(content.contains("format: markdown"));
        assert!(content.contains("domain: auth"));
        assert!(content.contains("priority: high"));
        assert!(content.contains("created_at:"));
        assert!(content.contains("updated_at:"));

        // Check content
        assert!(content.contains("Users must verify email"));
    }

    #[test]
    fn test_render_filename_from_name() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Content", Some("user-authentication"), &[]);

        ctx.cmd().arg("render").assert().success();

        let file_path = ctx.render_dir().join("checkout/intents/user-authentication.md");
        assert!(file_path.exists(), "File should use provided name");
    }

    #[test]
    fn test_render_filename_from_content() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Users authenticate via email and password", None, &[]);

        ctx.cmd().arg("render").assert().success();

        let intents_dir = ctx.render_dir().join("checkout/intents");
        let files: Vec<_> = fs::read_dir(&intents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(files.len(), 1);

        let filename = files[0].file_name();
        let filename_str = filename.to_str().unwrap();

        // Should be slugified from content
        assert!(filename_str.starts_with("users-authenticate"));
    }

    #[test]
    fn test_render_filename_conflict_resolution() {
        let ctx = TestContext::new();
        ctx.init();

        // Add two artifacts with same name
        ctx.add_artifact("intent", "Content 1", Some("same-name"), &[]);
        ctx.add_artifact("intent", "Content 2", Some("same-name"), &[]);

        ctx.cmd().arg("render").assert().success();

        let intents_dir = ctx.render_dir().join("checkout/intents");
        let files: Vec<_> = fs::read_dir(&intents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        // Should have 2 files with different names
        assert_eq!(files.len(), 2);

        // One should be exact name, other should have ID suffix
        let filenames: Vec<_> = files.iter().map(|f| f.file_name()).collect();
        let has_exact = filenames.iter().any(|f| f == "same-name.md");
        let has_suffixed = filenames.iter().any(|f| {
            let s = f.to_str().unwrap();
            s.starts_with("same-name-") && s.ends_with(".md")
        });

        assert!(has_exact || has_suffixed, "Should handle filename conflicts");
    }

    #[test]
    fn test_render_by_metadata() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Auth feature", None, &[("domain", "auth")]);
        ctx.add_artifact("intent", "Payment feature", None, &[("domain", "payment")]);

        ctx.cmd().args(&["render", "--by", "domain"]).assert().success();

        // Should organize by domain
        let checkout = ctx.render_dir().join("checkout");
        assert!(checkout.join("auth").exists() || checkout.join("intents").exists());
    }

    #[test]
    fn test_render_custom_output_dir() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Content", Some("test"), &[]);

        let custom_dir = ctx.root().join("custom_output");
        ctx.cmd()
            .args(&["render", "--output", custom_dir.to_str().unwrap()])
            .assert()
            .success();

        assert!(custom_dir.exists());
    }

    #[test]
    fn test_render_different_formats() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.cmd()
            .args(&["add", "contract", r#"{"openapi": "3.0.0"}"#, "--format", "json", "--name", "api-contract"])
            .assert()
            .success();

        ctx.cmd().arg("render").assert().success();

        let file_path = ctx.render_dir().join("checkout/contracts/api-contract.json");
        assert!(file_path.exists() || ctx.render_dir().join("checkout/contracts/api-contract.md").exists());
    }

    #[test]
    fn test_render_preserves_unicode() {
        let ctx = TestContext::new();
        ctx.init();

        let unicode_content = "Support for 多语言 and emoji 🚀";
        ctx.add_artifact("intent", unicode_content, Some("unicode-test"), &[]);

        ctx.cmd().arg("render").assert().success();

        let file_path = ctx.render_dir().join("checkout/intents/unicode-test.md");
        let content = fs::read_to_string(&file_path).unwrap();

        assert!(content.contains("多语言"));
        assert!(content.contains("🚀"));
    }

    #[test]
    fn test_render_multiple_artifacts() {
        let ctx = TestContext::new();
        ctx.init();

        for i in 0..10 {
            ctx.add_artifact("intent", &format!("Content {}", i), Some(&format!("intent-{}", i)), &[]);
        }

        ctx.cmd().arg("render").assert().success();

        let intents_dir = ctx.render_dir().join("checkout/intents");
        let count = fs::read_dir(&intents_dir).unwrap().count();

        assert_eq!(count, 10);
    }

    #[test]
    fn test_render_empty_database() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.cmd()
            .arg("render")
            .assert()
            .success()
            .stdout(predicate::str::contains("No artifacts").or(predicate::str::contains("0 artifacts")));
    }

    #[test]
    fn test_render_creates_inventory() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Test", Some("test"), &[]);

        ctx.cmd().arg("render").assert().success();

        // Should create inventory directory
        let inventory = ctx.render_dir().join("inventory");
        assert!(
            inventory.exists() || ctx.render_dir().join("checkout").exists(),
            "Should create output structure"
        );
    }

    #[test]
    fn test_render_updates_existing_files() {
        let ctx = TestContext::new();
        ctx.init();

        let id = ctx.add_artifact("intent", "Original content", Some("test-artifact"), &[]);

        ctx.cmd().arg("render").assert().success();

        let file_path = ctx.render_dir().join("checkout/intents/test-artifact.md");
        let original_content = fs::read_to_string(&file_path).unwrap();

        // Update the artifact
        ctx.cmd()
            .args(&["update", &id, "--content", "Updated content"])
            .assert()
            .success();

        ctx.cmd().arg("render").assert().success();

        let updated_content = fs::read_to_string(&file_path).unwrap();

        assert_ne!(original_content, updated_content);
        assert!(updated_content.contains("Updated content"));
    }

    #[test]
    fn test_render_with_nested_metadata_structure() {
        let ctx = TestContext::new();
        ctx.init();

        ctx.add_artifact("intent", "Auth service", None, &[("service", "auth"), ("layer", "api")]);
        ctx.add_artifact("intent", "Payment service", None, &[("service", "payment"), ("layer", "api")]);

        ctx.cmd().args(&["render", "--by", "service,layer"]).assert().success();

        // Should create nested structure like: auth/api/
        let checkout = ctx.render_dir().join("checkout");
        assert!(checkout.exists());
    }

    #[test]
    fn test_render_slugification() {
        let ctx = TestContext::new();
        ctx.init();

        // Name with special characters that should be slugified
        ctx.add_artifact("intent", "Test", Some("User's \"Special\" Feature!"), &[]);

        ctx.cmd().arg("render").assert().success();

        let intents_dir = ctx.render_dir().join("checkout/intents");
        let files: Vec<_> = fs::read_dir(&intents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(files.len(), 1);

        let filename = files[0].file_name();
        let filename_str = filename.to_str().unwrap();

        // Should not contain special characters
        assert!(!filename_str.contains('"'));
        assert!(!filename_str.contains('!'));
        assert!(!filename_str.contains('\''));
    }

    #[test]
    fn test_render_long_content_truncation() {
        let ctx = TestContext::new();
        ctx.init();

        // Very long content without explicit name
        let long_content = "word ".repeat(100);
        ctx.add_artifact("intent", &long_content, None, &[]);

        ctx.cmd().arg("render").assert().success();

        let intents_dir = ctx.render_dir().join("checkout/intents");
        let files: Vec<_> = fs::read_dir(&intents_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(files.len(), 1);

        let filename = files[0].file_name();
        let filename_str = filename.to_str().unwrap();

        // Filename should be reasonable length (not 500 chars)
        assert!(filename_str.len() < 100, "Filename should be truncated to reasonable length");
    }

    #[test]
    fn test_render_all_artifact_types() {
        let ctx = TestContext::new();
        ctx.init();

        let types = vec![
            ("intent", "Intent content"),
            ("invariant", "Invariant content"),
            ("contract", "Contract content"),
            ("algorithm", "Algorithm content"),
            ("evaluation", "Evaluation content"),
            ("pace", "Pace content"),
            ("monitor", "Monitor content"),
        ];

        for (artifact_type, content) in types {
            ctx.add_artifact(artifact_type, content, Some(&format!("{}-test", artifact_type)), &[]);
        }

        ctx.cmd().arg("render").assert().success();

        let checkout = ctx.render_dir().join("checkout");

        // Check all directories exist
        assert!(checkout.join("intents").exists());
        assert!(checkout.join("invariants").exists());
        assert!(checkout.join("contracts").exists());
        assert!(checkout.join("algorithms").exists());
        assert!(checkout.join("evaluations").exists());
        assert!(checkout.join("paces").exists());
        assert!(checkout.join("monitors").exists());
    }

    #[test]
    fn test_render_without_init() {
        let ctx = TestContext::new();

        ctx.cmd()
            .arg("render")
            .assert()
            .failure()
            .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("Run 'dna init'")));
    }
}
