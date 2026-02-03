use dna::embedding::EmbeddingProvider;
use dna::services::{ArtifactService, ConfigService, ContentFormat, KindService, SearchFilters};
use dna::testing::{TestDatabase, TestEmbedding};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

// -- Tests --

#[test]
fn init_intent_flow_registers_seven_kinds() {
    let temp_dir = TempDir::new().unwrap();
    let svc = ConfigService::new(temp_dir.path());

    let config = svc.init_intent_flow().unwrap();

    assert_eq!(config.kinds.definitions.len(), 7);
    for slug in &[
        "intent",
        "invariant",
        "contract",
        "algorithm",
        "evaluation",
        "pace",
        "monitor",
    ] {
        assert!(config.kinds.has(slug), "Missing kind: {}", slug);
    }
}

#[test]
fn init_intent_flow_persists_and_reloads() {
    let temp_dir = TempDir::new().unwrap();
    let svc = ConfigService::new(temp_dir.path());
    svc.init_intent_flow().unwrap();

    let reloaded = svc.load().unwrap();
    assert_eq!(reloaded.kinds.definitions.len(), 7);
    assert!(reloaded.kinds.has("evaluation"));
    assert_eq!(
        reloaded.kinds.get("evaluation").unwrap().description,
        "Success criteria, thresholds, and verification mechanisms"
    );
}

#[test]
fn init_intent_flow_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let svc = ConfigService::new(temp_dir.path());
    svc.init_intent_flow().unwrap();
    let config = svc.init_intent_flow().unwrap();
    assert_eq!(config.kinds.definitions.len(), 7);
}

#[test]
fn add_and_remove_custom_kind() {
    let temp_dir = TempDir::new().unwrap();
    let svc = ConfigService::new(temp_dir.path());
    svc.init().unwrap();

    assert!(svc.add_kind("deployment", "Deploy constraints").unwrap());
    assert!(svc.load().unwrap().kinds.has("deployment"));

    assert!(!svc.add_kind("deployment", "duplicate").unwrap());

    assert!(svc.remove_kind("deployment").unwrap());
    assert!(!svc.load().unwrap().kinds.has("deployment"));

    assert!(!svc.remove_kind("deployment").unwrap());
}

#[tokio::test]
async fn kind_service_scopes_by_kind() {
    let db = Arc::new(TestDatabase::new());
    let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);

    let artifact_svc = ArtifactService::new(db.clone(), embedding.clone());

    artifact_svc
        .add(
            "evaluation".into(),
            "eval 1".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();
    artifact_svc
        .add(
            "intent".into(),
            "intent 1".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();
    artifact_svc
        .add(
            "evaluation".into(),
            "eval 2".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    let eval_svc = KindService::new("evaluation".into(), db.clone(), embedding.clone());
    assert_eq!(eval_svc.list(None).await.unwrap().len(), 2);

    let intent_svc = KindService::new("intent".into(), db.clone(), embedding.clone());
    assert_eq!(intent_svc.list(None).await.unwrap().len(), 1);
}

#[tokio::test]
async fn kind_service_search_scoped() {
    let db = Arc::new(TestDatabase::new());
    let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);

    let artifact_svc = ArtifactService::new(db.clone(), embedding.clone());
    artifact_svc
        .add(
            "evaluation".into(),
            "login success".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();
    artifact_svc
        .add(
            "intent".into(),
            "login purpose".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    let eval_svc = KindService::new("evaluation".into(), db, embedding);
    let results = eval_svc.search("login", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].artifact.kind, "evaluation");
}

#[tokio::test]
async fn full_intent_flow_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_svc = ConfigService::new(temp_dir.path());

    // 1. Init with intent-flow
    let config = config_svc.init_intent_flow().unwrap();
    assert_eq!(config.kinds.definitions.len(), 7);

    let db = Arc::new(TestDatabase::new());
    let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
    let artifact_svc = ArtifactService::new(db.clone(), embedding.clone());

    // 2. Add truth artifacts
    let intent = artifact_svc
        .add(
            "intent".into(),
            "Users authenticate via email/password".into(),
            ContentFormat::Markdown,
            Some("user-auth".into()),
            HashMap::from([("domain".into(), "auth".into())]),
            None,
        )
        .await
        .unwrap();

    let invariant = artifact_svc
        .add(
            "invariant".into(),
            "Passwords hashed with bcrypt".into(),
            ContentFormat::Markdown,
            Some("password-hash".into()),
            HashMap::from([("domain".into(), "auth".into())]),
            None,
        )
        .await
        .unwrap();

    let evaluation = artifact_svc
        .add(
            "evaluation".into(),
            "Given valid credentials, when login, then token returned".into(),
            ContentFormat::Markdown,
            Some("login-eval".into()),
            HashMap::from([("domain".into(), "auth".into())]),
            None,
        )
        .await
        .unwrap();

    assert_eq!(intent.kind, "intent");
    assert_eq!(invariant.kind, "invariant");
    assert_eq!(evaluation.kind, "evaluation");
    assert!(intent.embedding.is_some());

    // 3. Kind-scoped queries
    let eval_svc = KindService::new("evaluation".into(), db.clone(), embedding.clone());
    assert_eq!(eval_svc.list(None).await.unwrap().len(), 1);

    // 4. Add custom kind
    config_svc.add_kind("deployment", "Deploy rules").unwrap();
    assert_eq!(config_svc.load().unwrap().kinds.definitions.len(), 8);

    artifact_svc
        .add(
            "deployment".into(),
            "Deploy to us-east-1 with 3 replicas".into(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    let deploy_svc = KindService::new("deployment".into(), db.clone(), embedding.clone());
    assert_eq!(deploy_svc.list(None).await.unwrap().len(), 1);

    // 5. Total artifacts count
    let all = artifact_svc.list(SearchFilters::default()).await.unwrap();
    assert_eq!(all.len(), 4);
}

#[test]
fn kind_tool_names_no_hyphens() {
    let kinds = ["evaluation", "pace", "my-custom-kind"];
    for kind in kinds {
        let prefix = kind.replace('-', "_");
        for action in &["search", "add", "list"] {
            let tool_name = format!("dna_{}_{}", prefix, action);
            assert!(
                !tool_name.contains('-'),
                "Tool name has hyphen: {}",
                tool_name
            );
            assert!(tool_name.starts_with("dna_"));
        }
    }
}

#[tokio::test]
async fn update_artifact_removes_label_with_empty_value() {
    let db = Arc::new(TestDatabase::new());
    let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
    let artifact_svc = ArtifactService::new(db.clone(), embedding.clone());

    // Create artifact with labels
    let artifact = artifact_svc
        .add(
            "intent".into(),
            "test content".into(),
            ContentFormat::Markdown,
            None,
            HashMap::from([
                ("env".into(), "production".into()),
                ("team".into(), "platform".into()),
            ]),
            None,
        )
        .await
        .unwrap();

    // Update with empty value for "env" label
    let updated = artifact_svc
        .update(
            &artifact.id,
            None,
            None,
            None,
            Some(HashMap::from([("env".into(), "".into())])),
            None,
        )
        .await
        .unwrap();

    // Verify "env" label is removed (not set to empty string)
    assert!(
        !updated.metadata.contains_key("env"),
        "Label 'env' should be removed when updated with empty value"
    );
    assert_eq!(
        updated.metadata.get("team"),
        Some(&"platform".to_string()),
        "Other labels should remain unchanged"
    );
}
