#![allow(deprecated)] // cargo_bin is deprecated but still functional

/// E2E integration tests for the kind workflow
///
/// Tests the complete kind management workflow: init --template, kind add/list/show/remove,
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

// -- Init with template tests --

#[test]
fn test_init_intent_template_registers_eleven_kinds() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'intent' applied"))
        .stdout(predicate::str::contains("intent"))
        .stdout(predicate::str::contains("contract"))
        .stdout(predicate::str::contains("algorithm"))
        .stdout(predicate::str::contains("evaluation"))
        .stdout(predicate::str::contains("pace"))
        .stdout(predicate::str::contains("monitor"))
        .stdout(predicate::str::contains("glossary"))
        .stdout(predicate::str::contains("integration"))
        .stdout(predicate::str::contains("reporting"))
        .stdout(predicate::str::contains("compliance"))
        .stdout(predicate::str::contains("constraint"));

    // Verify config file has kinds
    let config = std::fs::read_to_string(ctx.dna_dir().join("config.toml")).unwrap();
    assert!(config.contains("[kinds]") || config.contains("[[kinds.definitions]]"));
}

#[test]
fn test_init_intent_template_creates_config_with_kinds() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    let config = std::fs::read_to_string(ctx.dna_dir().join("config.toml")).unwrap();

    // Verify each kind is in the config
    for kind in [
        "intent",
        "contract",
        "algorithm",
        "evaluation",
        "pace",
        "monitor",
        "glossary",
        "integration",
        "reporting",
        "compliance",
        "constraint",
    ] {
        assert!(
            config.contains(&format!("slug = \"{}\"", kind)),
            "Config should contain kind: {}",
            kind
        );
    }
}

#[test]
fn test_init_ai_safety_template() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--template", "agentic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Template 'agentic' applied"))
        .stdout(predicate::str::contains("behavior"))
        .stdout(predicate::str::contains("boundary"))
        .stdout(predicate::str::contains("threat"))
        .stdout(predicate::str::contains("eval"))
        .stdout(predicate::str::contains("governance"));
}

#[test]
fn test_init_unknown_template_fails() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--template", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown template"));
}

#[test]
fn test_list_templates() {
    let ctx = TestContext::new();

    ctx.cmd()
        .args(["init", "--list-templates"])
        .assert()
        .success()
        .stdout(predicate::str::contains("intent"))
        .stdout(predicate::str::contains("agentic"));
}

// -- Kind management CLI tests --

#[test]
fn test_kind_list_shows_registered_kinds() {
    let ctx = TestContext::new();
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    ctx.cmd()
        .args(["kind", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Registered kinds"))
        .stdout(predicate::str::contains("intent"))
        .stdout(predicate::str::contains("evaluation"));
}

#[test]
fn test_kind_list_empty_without_template() {
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
        .args(["kind", "add", "My Custom Kind", "Custom kind for testing"])
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
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    ctx.cmd()
        .args(["kind", "add", "evaluation", "Test evaluation kind"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_kind_show_displays_operations() {
    let ctx = TestContext::new();
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

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
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

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
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

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
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

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
        .args(["kind", "add", "deployment", "Deployment artifacts"])
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
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

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
fn test_full_intent_template_workflow() {
    let ctx = TestContext::new();

    // Step 1: Initialize with intent template
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    // Step 2: Add truth artifacts for various kinds
    let artifacts = [
        ("intent", "Users can authenticate via email and password"),
        ("constraint", "Passwords must be hashed with bcrypt"),
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
        .args(["kind", "add", "deployment", "Deployment rules"])
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
fn test_kind_add_requires_description() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();

    // Add without description should fail
    ctx.cmd()
        .args(["kind", "add", "testing"])
        .assert()
        .failure();
}

// -- MCP tool naming convention tests --

#[test]
fn test_kind_show_displays_correct_mcp_tool_names() {
    let ctx = TestContext::new();
    ctx.cmd().args(["init"]).assert().success();
    ctx.cmd()
        .args([
            "kind",
            "add",
            "my-custom-kind",
            "Custom kind for MCP testing",
        ])
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

// -- Context flag tests --

#[test]
fn test_add_artifact_with_context() {
    let ctx = TestContext::new();
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    // Add artifact with context
    ctx.cmd()
        .args([
            "add",
            "intent",
            "User login flow",
            "--context",
            "Part of the authentication system. Related to GDPR compliance.",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added artifact"));

    // Verify the artifact was created (we can't easily verify the context embedding
    // without searching, but we can verify the command succeeded)
}

#[test]
fn test_update_artifact_context() {
    let ctx = TestContext::new();
    ctx.cmd()
        .args(["init", "--template", "intent"])
        .assert()
        .success();

    // Add artifact
    let output = ctx
        .cmd()
        .args(["add", "intent", "Test content"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Extract the ID from the output (it's in "Added artifact: <id>" line)
    let output_str = String::from_utf8_lossy(&output);
    let id = output_str
        .lines()
        .find(|l| l.starts_with("Added artifact:"))
        .and_then(|l| l.split(": ").nth(1))
        .unwrap()
        .trim();

    // Update with context
    ctx.cmd()
        .args(["update", id, "--context", "Now part of auth system"])
        .assert()
        .success();
}
