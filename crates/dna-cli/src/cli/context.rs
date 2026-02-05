use anyhow::Result;
use clap::Args;
use dna::services::{ArtifactService, ConfigService, SearchFilters};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct ContextArgs {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn execute(args: ContextArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;

    // Load artifact counts per kind
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;
    let service = ArtifactService::new(db, embedding);

    let artifacts = service.list(SearchFilters::default()).await?;
    let mut counts_by_kind: HashMap<String, usize> = HashMap::new();
    for artifact in &artifacts {
        *counts_by_kind.entry(artifact.kind.clone()).or_default() += 1;
    }

    if args.json {
        let kinds: Vec<serde_json::Value> = config
            .kinds
            .definitions
            .iter()
            .map(|k| {
                serde_json::json!({
                    "slug": k.slug,
                    "description": k.description,
                    "artifact_count": counts_by_kind.get(&k.slug).unwrap_or(&0),
                })
            })
            .collect();

        let labels: Vec<serde_json::Value> = config
            .labels
            .definitions
            .iter()
            .map(|l| {
                serde_json::json!({
                    "key": l.key,
                    "description": l.description,
                })
            })
            .collect();

        let output = serde_json::json!({
            "kinds": kinds,
            "labels": labels,
            "total_artifacts": artifacts.len(),
            "embedding_model": config.model.name,
        });

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Project Truth Schema");
        println!("====================");
        println!();

        let kinds = &config.kinds.definitions;
        println!("Kinds ({} registered):", kinds.len());
        for kind in kinds {
            let count = counts_by_kind.get(&kind.slug).unwrap_or(&0);
            println!(
                "  {:<16} {:<50} [{} artifacts]",
                kind.slug,
                truncate(&kind.description, 50),
                count
            );
        }

        println!();

        let labels = &config.labels.definitions;
        println!("Labels ({} registered):", labels.len());
        for label in labels {
            println!("  {:<16} {}", label.key, label.description);
        }

        println!();
        println!("Embedding model: {}", config.model.name);
        println!("Total artifacts: {}", artifacts.len());
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 3).collect();
        format!("{}...", truncated)
    }
}
