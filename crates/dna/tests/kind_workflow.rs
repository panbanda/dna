use anyhow::Result;
use dna::db::Database;
use dna::embedding::EmbeddingProvider;
use dna::services::{
    Artifact, ArtifactService, ConfigService, ContentFormat, KindService, SearchFilters,
    SearchResult,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

// -- Test doubles --

struct TestEmbedding;

#[async_trait::async_trait]
impl EmbeddingProvider for TestEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        Ok((0..384)
            .map(|i| ((hash.wrapping_add(i as u32) % 1000) as f32) / 1000.0)
            .collect())
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn model_id(&self) -> &str {
        "test-embedding-model"
    }

    fn dimensions(&self) -> usize {
        384
    }
}

struct TestDatabase {
    artifacts: Mutex<HashMap<String, Artifact>>,
}

impl TestDatabase {
    fn new() -> Self {
        Self {
            artifacts: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl Database for TestDatabase {
    async fn insert(&self, artifact: &Artifact) -> Result<()> {
        self.artifacts
            .lock()
            .unwrap()
            .insert(artifact.id.clone(), artifact.clone());
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Artifact>> {
        Ok(self.artifacts.lock().unwrap().get(id).cloned())
    }

    async fn update(&self, artifact: &Artifact) -> Result<()> {
        self.artifacts
            .lock()
            .unwrap()
            .insert(artifact.id.clone(), artifact.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.artifacts.lock().unwrap().remove(id).is_some())
    }

    async fn list(&self, filters: SearchFilters) -> Result<Vec<Artifact>> {
        let all: Vec<_> = self.artifacts.lock().unwrap().values().cloned().collect();
        Ok(all
            .into_iter()
            .filter(|a| filters.kind.as_ref().map_or(true, |k| &a.kind == k))
            .collect())
    }

    async fn search(
        &self,
        _query_embedding: &[f32],
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>> {
        let all: Vec<_> = self.artifacts.lock().unwrap().values().cloned().collect();
        Ok(all
            .into_iter()
            .filter(|a| filters.kind.as_ref().map_or(true, |k| &a.kind == k))
            .map(|a| SearchResult {
                artifact: a,
                score: 0.85,
            })
            .collect())
    }
}

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
