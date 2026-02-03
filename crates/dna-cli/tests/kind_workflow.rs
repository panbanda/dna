#![allow(deprecated)] // cargo_bin is deprecated but still functional

/// E2E integration tests for the kind workflow
///
/// Tests the complete kind management workflow: init --intent-flow, kind add/list/show/remove,
/// and kind-scoped operations (add, list, search)
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

    fn dna_dir(&self) -> PathBuf {
        self.root().join(".dna")
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("dna").unwrap();
        cmd.current_dir(self.root());
        cmd
    }
}

// -- Init with intent-flow tests --

#[test]
fn test_init_intent_flow_registers_seven_kinds() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--intent-flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Intent-flow kinds registered"))
        .stdout(predicate::str::contains("intent"))
        .stdout(predicate::str::contains("invariant"))
        .stdout(predicate::str::contains("contract"))
        .stdout(predicate::str::contains("algorithm"))
        .stdout(predicate::str::contains("evaluation"))
        .stdout(predicate::str::contains("pace"))
        .stdout(predicate::str::contains("monitor"));

    // Verify config file has kinds
    let config = std::fs::read_to_string(ctx.dna_dir().join("config.toml")).unwrap();
    assert!(config.contains("[kinds]") || config.contains("[[kinds.definitions]]"));
}

#[test]
fn test_init_intent_flow_creates_config_with_kinds() {
    let ctx = TestContext::new();

    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    let config = std::fs::read_to_string(ctx.dna_dir().join("config.toml")).unwrap();

    // Verify each kind is in the config
    for kind in [
        "intent",
        "invariant",
        "contract",
        "algorithm",
        "evaluation",
        "pace",
        "monitor",
    ] {
        assert!(
            config.contains(&format!("slug = \"{}\"", kind)),
            "Config should contain kind: {}",
            kind
        );
    }
}

// -- Kind management CLI tests --

#[test]
fn test_kind_list_shows_registered_kinds() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Registered kinds"))
        .stdout(predicate::str::contains("intent"))
        .stdout(predicate::str::contains("evaluation"));
}

#[test]
fn test_kind_list_empty_without_intent_flow() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No kinds registered"));
}

#[test]
fn test_kind_add_custom() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    ctx.cmd()
        .args([
            "kind",
            "add",
            "deployment",
            "--description",
            "Deployment configuration and constraints",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added kind: deployment"))
        .stdout(predicate::str::contains("Deployment configuration"));

    // Verify it appears in list
    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deployment"));
}

#[test]
fn test_kind_add_slugifies_name() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    ctx.cmd()
        .args(["kind", "add", "My Custom Kind"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added kind: my-custom-kind"));

    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-custom-kind"));
}

#[test]
fn test_kind_add_duplicate_reports_exists() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    ctx.cmd()
        .args(["kind", "add", "evaluation"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_kind_show_displays_operations() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    ctx.cmd()
        .args(["kind", "show", "evaluation"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kind: evaluation"))
        .stdout(predicate::str::contains("CLI:"))
        .stdout(predicate::str::contains("dna add evaluation"))
        .stdout(predicate::str::contains("API:"))
        .stdout(predicate::str::contains("/api/v1/kinds/evaluation"))
        .stdout(predicate::str::contains("MCP tools:"))
        .stdout(predicate::str::contains("dna_evaluation_search"));
}

#[test]
fn test_kind_show_not_found() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    ctx.cmd()
        .args(["kind", "show", "nonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn test_kind_remove_warns_without_force() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    // Without --force, should warn about orphaned artifacts
    ctx.cmd()
        .args(["kind", "remove", "monitor"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Warning"))
        .stderr(predicate::str::contains("orphaned"));

    // Kind should still exist
    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("monitor"));
}

#[test]
fn test_kind_remove_with_force() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    ctx.cmd()
        .args(["kind", "remove", "monitor", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed kind: monitor"));

    // Verify it's gone
    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("monitor").not());
}

#[test]
fn test_kind_remove_nonexistent() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    // Nonexistent kind reports not found (doesn't need --force)
    ctx.cmd()
        .args(["kind", "remove", "nonexistent", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

// -- Kind-scoped artifact operations --

#[test]
fn test_add_artifact_with_registered_kind() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    ctx.cmd()
        .args([
            "add",
            "evaluation",
            "Given valid user, when login, then session created",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added artifact"));
}

#[test]
fn test_add_artifact_with_custom_kind() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();
    ctx.cmd()
        .args(["kind", "add", "deployment"])
        .assert()
        .success();

    ctx.cmd()
        .args([
            "add",
            "deployment",
            "Must deploy to us-east-1 with 3 replicas",
        ])
        .assert()
        .success();
}

#[test]
fn test_list_filters_by_kind() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    // Add artifacts of different kinds
    ctx.cmd()
        .args(["add", "intent", "User authentication via email"])
        .assert()
        .success();
    ctx.cmd()
        .args(["add", "evaluation", "Login returns valid token"])
        .assert()
        .success();
    ctx.cmd()
        .args(["add", "intent", "User can reset password"])
        .assert()
        .success();

    // List only intent artifacts - should show 2 artifacts
    ctx.cmd()
        .args(["list", "--kind", "intent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 artifacts"))
        .stdout(predicate::str::contains("intent"));
}

// -- Full workflow tests --

#[test]
fn test_full_intent_flow_workflow() {
    let ctx = TestContext::new();

    // Step 1: Initialize with intent-flow
    ctx.cmd().args(["init", "--intent-flow"]).assert().success();

    // Step 2: Add truth artifacts for each kind
    let artifacts = [
        ("intent", "Users can authenticate via email and password"),
        ("invariant", "Passwords must be hashed with bcrypt"),
        ("contract", "POST /auth/login returns JWT token"),
        (
            "algorithm",
            "Password hashing uses bcrypt with cost factor 12",
        ),
        (
            "evaluation",
            "Given valid credentials, when login, then token returned",
        ),
        ("pace", "Auth API changes require 2-week deprecation notice"),
        ("monitor", "Auth endpoint P99 latency < 200ms"),
    ];

    for (kind, content) in &artifacts {
        ctx.cmd().args(["add", kind, content]).assert().success();
    }

    // Step 3: Verify all kinds have artifacts
    for (kind, _) in &artifacts {
        ctx.cmd().args(["list", "--kind", kind]).assert().success();
    }

    // Step 4: Add custom kind
    ctx.cmd()
        .args([
            "kind",
            "add",
            "deployment",
            "--description",
            "Deployment rules",
        ])
        .assert()
        .success();

    ctx.cmd()
        .args([
            "add",
            "deployment",
            "Deploy to us-east-1 with min 3 replicas",
        ])
        .assert()
        .success();

    // Step 5: Search across kinds
    ctx.cmd().args(["search", "auth"]).assert().success();
}

#[test]
fn test_kind_without_init_fails() {
    let ctx = TestContext::new();

    ctx.cmd().args(["kind", "list"]).assert().failure().stderr(
        predicate::str::contains("not initialized").or(predicate::str::contains("Run 'dna init'")),
    );
}

#[test]
fn test_kind_add_default_description() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    // Add without description
    ctx.cmd()
        .args(["kind", "add", "testing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("testing artifacts"));
}

// -- MCP tool naming convention tests --

#[test]
fn test_kind_show_displays_correct_mcp_tool_names() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();
    ctx.cmd()
        .args(["kind", "add", "my-custom-kind"])
        .assert()
        .success();

    // MCP tools should use underscores, not hyphens
    ctx.cmd()
        .args(["kind", "show", "my-custom-kind"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dna_my_custom_kind_search"))
        .stdout(predicate::str::contains("dna_my_custom_kind_add"))
        .stdout(predicate::str::contains("dna_my_custom_kind_list"));
}
