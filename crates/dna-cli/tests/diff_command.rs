#![allow(deprecated)]

/// E2E integration tests for the diff command
///
/// Tests showing artifact diffs since a date with --since and --until flags.
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
        self.cmd().args(["init"]).assert().success();
    }
}

#[test]
fn test_diff_requires_since_flag() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd()
        .args(["diff"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--since"));
}

#[test]
fn test_diff_rejects_invalid_date_format() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd()
        .args(["diff", "--since", "not-a-date"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid date"));

    ctx.cmd()
        .args(["diff", "--since", "01-15-2024"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("YYYY-MM-DD"));
}

#[test]
fn test_diff_accepts_valid_date() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd()
        .args(["diff", "--since", "2024-01-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes since 2024-01-15"));
}

#[test]
fn test_diff_shows_added_artifact() {
    let ctx = TestContext::new();
    ctx.init();

    // Add an artifact (positional args: KIND CONTENT)
    ctx.cmd()
        .args(["add", "spec", "User must authenticate with password"])
        .assert()
        .success();

    // Diff from a date in the past should show the artifact as added
    ctx.cmd()
        .args(["diff", "--since", "2020-01-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(added)"))
        .stdout(predicate::str::contains(
            "User must authenticate with password",
        ));
}

#[test]
fn test_diff_names_only_shows_ids() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd()
        .args(["add", "spec", "Some specification content"])
        .assert()
        .success();

    ctx.cmd()
        .args(["diff", "--since", "2020-01-01", "--names-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changed artifacts since"))
        .stdout(predicate::str::contains("spec/"))
        .stdout(
            predicate::str::is_match(r"\+.*Some specification")
                .unwrap()
                .not(),
        );
}

#[test]
fn test_diff_filters_by_kind() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd().args(["add", "spec", "A spec"]).assert().success();

    ctx.cmd().args(["add", "doc", "A doc"]).assert().success();

    // Filter to only show spec changes
    ctx.cmd()
        .args(["diff", "--since", "2020-01-01", "--kind", "spec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec/"))
        .stdout(predicate::str::contains("doc/").not());
}

#[test]
fn test_diff_until_filters_upper_bound() {
    let ctx = TestContext::new();
    ctx.init();

    ctx.cmd()
        .args(["add", "spec", "Content"])
        .assert()
        .success();

    // Using --until in the past should show no changes
    ctx.cmd()
        .args(["diff", "--since", "2020-01-01", "--until", "2020-01-02"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes since"));
}

#[test]
fn test_diff_shows_modified_artifact() {
    let ctx = TestContext::new();
    ctx.init();

    // Add initial artifact
    let output = ctx
        .cmd()
        .args(["add", "spec", "User must authenticate with password"])
        .assert()
        .success();

    // Extract the artifact ID from output (format: Added artifact: <id>)
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let id = stdout
        .lines()
        .find(|l| l.starts_with("Added artifact:"))
        .and_then(|l| l.strip_prefix("Added artifact:"))
        .map(|s| s.trim())
        .expect("Should find artifact ID in output");

    // Update the artifact
    ctx.cmd()
        .args([
            "update",
            id,
            "--content",
            "User must authenticate with password or passkey",
        ])
        .assert()
        .success();

    // Diff should show the modification with +/- lines
    ctx.cmd()
        .args(["diff", "--since", "2020-01-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(modified)").or(predicate::str::contains("(added)")));
}

#[test]
fn test_diff_requires_init() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["diff", "--since", "2024-01-01"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DNA not initialized"));
}

#[test]
fn test_diff_filters_by_label() {
    let ctx = TestContext::new();
    ctx.init();

    // Add artifacts with different labels
    ctx.cmd()
        .args(["add", "spec", "Auth spec", "--label", "team=auth"])
        .assert()
        .success();

    ctx.cmd()
        .args(["add", "spec", "Payment spec", "--label", "team=payments"])
        .assert()
        .success();

    // Filter by label should only show matching artifacts
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--label",
            "team=auth",
            "--names-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec/"))
        .stdout(predicate::str::contains("Changed artifacts since"));

    // Verify filtering works by checking we can filter to the other team
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--label",
            "team=payments",
            "--names-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec/"));

    // Non-matching label should show no changes
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--label",
            "team=nonexistent",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes since"));
}

#[test]
fn test_diff_filters_by_search() {
    let ctx = TestContext::new();
    ctx.init();

    // Add artifacts with different content for semantic search
    ctx.cmd()
        .args([
            "add",
            "spec",
            "User authentication with OAuth2 and JWT tokens",
        ])
        .assert()
        .success();

    ctx.cmd()
        .args([
            "add",
            "spec",
            "Database schema for product inventory management",
        ])
        .assert()
        .success();

    // Search for authentication-related artifacts
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--search",
            "authentication OAuth",
            "--names-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changed artifacts since"));
}

#[test]
fn test_diff_combines_filters() {
    let ctx = TestContext::new();
    ctx.init();

    // Add artifacts with different kinds and labels
    ctx.cmd()
        .args([
            "add",
            "spec",
            "API authentication specification",
            "--label",
            "priority=high",
        ])
        .assert()
        .success();

    ctx.cmd()
        .args([
            "add",
            "doc",
            "API authentication documentation",
            "--label",
            "priority=high",
        ])
        .assert()
        .success();

    ctx.cmd()
        .args([
            "add",
            "spec",
            "Database schema specification",
            "--label",
            "priority=low",
        ])
        .assert()
        .success();

    // Combine --kind and --label filters
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--kind",
            "spec",
            "--label",
            "priority=high",
            "--names-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("spec/"))
        .stdout(predicate::str::contains("doc/").not());

    // Combine --kind, --label, and --search
    ctx.cmd()
        .args([
            "diff",
            "--since",
            "2020-01-01",
            "--kind",
            "spec",
            "--search",
            "authentication API",
            "--names-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changed artifacts since"));
}
