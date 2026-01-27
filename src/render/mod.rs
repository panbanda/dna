use crate::services::Artifact;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Service for rendering artifacts to filesystem
pub struct RenderService {
    output_dir: PathBuf,
}

impl RenderService {
    /// Create a new render service
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Render all artifacts to files
    pub async fn render_all(&self, artifacts: &[Artifact], group_by: &[String]) -> Result<()> {
        // Group artifacts
        let groups = self.group_artifacts(artifacts, group_by);

        for (path_parts, group_artifacts) in groups {
            for artifact in group_artifacts {
                self.render_artifact(artifact, &path_parts).await?;
            }
        }

        Ok(())
    }

    /// Group artifacts by metadata keys
    fn group_artifacts<'a>(
        &self,
        artifacts: &'a [Artifact],
        keys: &[String],
    ) -> HashMap<Vec<String>, Vec<&'a Artifact>> {
        let mut groups: HashMap<Vec<String>, Vec<&'a Artifact>> = HashMap::new();

        for artifact in artifacts {
            let mut path_parts = vec![artifact.artifact_type.to_string()];

            for key in keys {
                if let Some(value) = artifact.metadata.get(key) {
                    path_parts.push(value.clone());
                }
            }

            groups.entry(path_parts).or_default().push(artifact);
        }

        groups
    }

    /// Render a single artifact to file
    async fn render_artifact(&self, artifact: &Artifact, path_parts: &[String]) -> Result<()> {
        // Build directory path
        let mut dir_path = self.output_dir.clone();
        for part in path_parts {
            dir_path.push(part);
        }
        tokio::fs::create_dir_all(&dir_path).await?;

        // Generate filename
        let filename = self.generate_filename(artifact)?;
        let file_path = dir_path.join(filename);

        // Generate frontmatter
        let frontmatter = self.generate_frontmatter(artifact)?;

        // Write file
        let content = format!("---\n{}\n---\n\n{}", frontmatter, artifact.content);
        tokio::fs::write(&file_path, content).await?;

        Ok(())
    }

    /// Generate filename for artifact
    fn generate_filename(&self, artifact: &Artifact) -> Result<String> {
        let extension = artifact.file_extension();

        // Try using name first
        if let Some(name) = &artifact.name {
            return Ok(format!("{}.{}", slug::slugify(name), extension));
        }

        // Otherwise, slugify first 50 chars of content
        let slug_text = artifact.content.chars().take(50).collect::<String>();
        let slug = slug::slugify(&slug_text);

        if !slug.is_empty() {
            Ok(format!("{}.{}", slug, extension))
        } else {
            // Last resort: use ID
            Ok(format!("{}.{}", artifact.id, extension))
        }
    }

    /// Generate YAML frontmatter
    fn generate_frontmatter(&self, artifact: &Artifact) -> Result<String> {
        let mut frontmatter = format!(
            "id: {}\ntype: {}\nformat: {}",
            artifact.id, artifact.artifact_type, artifact.format
        );

        if !artifact.metadata.is_empty() {
            frontmatter.push_str("\nmetadata:");
            for (key, value) in &artifact.metadata {
                frontmatter.push_str(&format!("\n  {}: {}", key, value));
            }
        }

        frontmatter.push_str(&format!(
            "\ncreated_at: {}\nupdated_at: {}",
            artifact.created_at.to_rfc3339(),
            artifact.updated_at.to_rfc3339()
        ));

        Ok(frontmatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{ArtifactType, ContentFormat};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_artifact(
        name: Option<&str>,
        content: &str,
        artifact_type: ArtifactType,
        metadata: HashMap<String, String>,
    ) -> Artifact {
        use crate::services::Artifact;
        Artifact::new(
            artifact_type,
            content.to_string(),
            ContentFormat::Markdown,
            name.map(String::from),
            metadata,
            "test-model".to_string(),
        )
    }

    #[test]
    fn generate_filename_uses_name_when_present() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let artifact = create_test_artifact(
            Some("Test Artifact Name"),
            "content",
            ArtifactType::Intent,
            HashMap::new(),
        );

        let filename = service.generate_filename(&artifact).unwrap();
        assert_eq!(filename, "test-artifact-name.md");
    }

    #[test]
    fn generate_filename_uses_content_when_no_name() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let artifact = create_test_artifact(
            None,
            "This is the artifact content that should be slugified",
            ArtifactType::Intent,
            HashMap::new(),
        );

        let filename = service.generate_filename(&artifact).unwrap();
        assert!(filename.contains("this-is-the-artifact-content"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn generate_filename_uses_id_as_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let artifact = create_test_artifact(None, "!@#$%^", ArtifactType::Intent, HashMap::new());

        let filename = service.generate_filename(&artifact).unwrap();
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn generate_frontmatter_includes_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let artifact = create_test_artifact(
            Some("test"),
            "content",
            ArtifactType::Contract,
            HashMap::new(),
        );

        let frontmatter = service.generate_frontmatter(&artifact).unwrap();
        assert!(frontmatter.contains(&format!("id: {}", artifact.id)));
        assert!(frontmatter.contains("type: contract"));
        assert!(frontmatter.contains("format: markdown"));
        assert!(frontmatter.contains("created_at:"));
        assert!(frontmatter.contains("updated_at:"));
    }

    #[test]
    fn generate_frontmatter_includes_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), "auth".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let artifact =
            create_test_artifact(Some("test"), "content", ArtifactType::Intent, metadata);

        let frontmatter = service.generate_frontmatter(&artifact).unwrap();
        assert!(frontmatter.contains("metadata:"));
        assert!(frontmatter.contains("domain: auth"));
        assert!(frontmatter.contains("priority: high"));
    }

    #[test]
    fn group_artifacts_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let a1 = create_test_artifact(None, "intent1", ArtifactType::Intent, HashMap::new());
        let a2 = create_test_artifact(None, "contract1", ArtifactType::Contract, HashMap::new());
        let a3 = create_test_artifact(None, "intent2", ArtifactType::Intent, HashMap::new());

        let artifacts = vec![a1, a2, a3];
        let groups = service.group_artifacts(&artifacts, &[]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[&vec!["intent".to_string()]].len(), 2);
        assert_eq!(groups[&vec!["contract".to_string()]].len(), 1);
    }

    #[test]
    fn group_artifacts_by_metadata_key() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let mut meta1 = HashMap::new();
        meta1.insert("domain".to_string(), "auth".to_string());

        let mut meta2 = HashMap::new();
        meta2.insert("domain".to_string(), "billing".to_string());

        let a1 = create_test_artifact(None, "one", ArtifactType::Intent, meta1);
        let a2 = create_test_artifact(None, "two", ArtifactType::Intent, meta2);

        let artifacts = vec![a1, a2];
        let groups = service.group_artifacts(&artifacts, &["domain".to_string()]);

        assert_eq!(groups.len(), 2);
    }

    #[tokio::test]
    async fn render_artifact_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let artifact = create_test_artifact(
            Some("test-artifact"),
            "This is the content",
            ArtifactType::Intent,
            HashMap::new(),
        );

        service
            .render_artifact(&artifact, &["intent".to_string()])
            .await
            .unwrap();

        let file_path = temp_dir.path().join("intent").join("test-artifact.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("This is the content"));
        assert!(content.contains(&artifact.id));
    }

    #[tokio::test]
    async fn render_all_creates_grouped_files() {
        let temp_dir = TempDir::new().unwrap();
        let service = RenderService::new(temp_dir.path().to_path_buf());

        let a1 = create_test_artifact(
            Some("intent-one"),
            "content1",
            ArtifactType::Intent,
            HashMap::new(),
        );
        let a2 = create_test_artifact(
            Some("contract-one"),
            "content2",
            ArtifactType::Contract,
            HashMap::new(),
        );

        let artifacts = vec![a1, a2];
        service.render_all(&artifacts, &[]).await.unwrap();

        assert!(temp_dir
            .path()
            .join("intent")
            .join("intent-one.md")
            .exists());
        assert!(temp_dir
            .path()
            .join("contract")
            .join("contract-one.md")
            .exists());
    }
}
