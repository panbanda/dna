use super::types::ProjectConfig;
use anyhow::{Context, Result};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use std::path::{Path, PathBuf};

/// Service for configuration management
pub struct ConfigService {
    config_path: PathBuf,
}

impl ConfigService {
    /// Create a new config service
    pub fn new(project_root: &Path) -> Self {
        let config_path = project_root.join(".dna").join("config.toml");
        Self { config_path }
    }

    /// Initialize configuration with defaults
    pub fn init(&self) -> Result<ProjectConfig> {
        let config = ProjectConfig::default();
        self.save(&config)?;
        Ok(config)
    }

    /// Load configuration from file, with env var overrides (DNA_ prefix, __ separator)
    pub fn load(&self) -> Result<ProjectConfig> {
        let mut figment = Figment::from(Serialized::defaults(ProjectConfig::default()));

        if self.config_path.exists() {
            figment = figment.merge(Toml::file(&self.config_path));
        }

        figment = figment.merge(Env::prefixed("DNA_").split("__"));

        let config: ProjectConfig = figment.extract().context("Failed to load configuration")?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config: &ProjectConfig) -> Result<()> {
        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        std::fs::write(&self.config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    /// Update model configuration
    pub fn update_model(&self, provider: String, name: String) -> Result<()> {
        let mut config = self.load()?;
        config.model.provider = provider;
        config.model.name = name;
        self.save(&config)?;
        Ok(())
    }

    /// Get a configuration value by dotted key
    pub fn get(&self, key: &str) -> Result<String> {
        let config = self.load()?;
        match key {
            "model.provider" => Ok(config.model.provider),
            "model.name" => Ok(config.model.name),
            "model.api_key" => Ok(config.model.api_key.unwrap_or_default()),
            "model.base_url" => Ok(config.model.base_url.unwrap_or_default()),
            "storage.uri" => Ok(config.storage.uri.unwrap_or_default()),
            _ => Err(anyhow::anyhow!("Unknown config key: {}", key)),
        }
    }

    /// Set a configuration value by dotted key
    pub fn set(&self, key: &str, value: String) -> Result<()> {
        let mut config = self.load()?;
        match key {
            "model.provider" => config.model.provider = value,
            "model.name" => config.model.name = value,
            "model.api_key" => config.model.api_key = Some(value),
            "model.base_url" => config.model.base_url = Some(value),
            "storage.uri" => config.storage.uri = Some(value),
            _ => return Err(anyhow::anyhow!("Unknown config key: {}", key)),
        }
        self.save(&config)?;
        Ok(())
    }

    /// Resolve the storage URI from config, defaulting to local path
    pub fn resolve_storage_uri(&self, project_root: &Path) -> Result<String> {
        let config = self.load()?;
        let uri = match &config.storage.uri {
            Some(uri) if uri.starts_with("s3://") => uri.clone(),
            Some(uri) => project_root.join(uri).to_string_lossy().to_string(),
            None => project_root
                .join(".dna")
                .join("db")
                .join("artifacts.lance")
                .to_string_lossy()
                .to_string(),
        };
        Ok(uri)
    }

    /// Initialize with the intent-flow artifact kinds from the intent-starter pattern
    pub fn init_intent_flow(&self) -> Result<ProjectConfig> {
        let mut config = if self.exists() {
            self.load()?
        } else {
            ProjectConfig::default()
        };

        let intent_kinds = [
            (
                "intent",
                "Core purpose statements expressing why features exist",
            ),
            (
                "invariant",
                "Non-negotiable properties that must always hold true",
            ),
            (
                "contract",
                "Guaranteed observable behavior and API specifications",
            ),
            (
                "algorithm",
                "How critical operations work for security and performance",
            ),
            (
                "evaluation",
                "Success criteria, thresholds, and verification mechanisms",
            ),
            ("pace", "Velocity constraints and change governance rules"),
            (
                "monitor",
                "Required observability and monitoring guarantees",
            ),
        ];

        for (slug, description) in intent_kinds {
            config.kinds.add(slug.to_string(), description.to_string());
        }

        self.save(&config)?;
        Ok(config)
    }

    /// Add a kind to the config.
    ///
    /// Validates the slug before adding. Returns an error if the slug is invalid.
    /// Returns Ok(false) if the kind already exists.
    pub fn add_kind(&self, slug: &str, description: &str) -> Result<bool> {
        // Validate slug before adding
        super::validate_kind_slug(slug)?;

        let mut config = self.load()?;
        let added = config.kinds.add(slug.to_string(), description.to_string());
        if added {
            self.save(&config)?;
        }
        Ok(added)
    }

    /// Remove a kind from the config
    pub fn remove_kind(&self, slug: &str) -> Result<bool> {
        let mut config = self.load()?;
        let removed = config.kinds.remove(slug);
        if removed {
            self.save(&config)?;
        }
        Ok(removed)
    }

    /// Check if configuration exists
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn init_creates_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());

        let config = service.init().unwrap();
        assert_eq!(config.model.provider, "local");
        assert_eq!(config.model.name, "BAAI/bge-small-en-v1.5");
    }

    #[test]
    fn init_creates_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        assert!(service.exists());
    }

    #[test]
    fn load_returns_saved_config() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        let init_config = service.init().unwrap();
        let loaded = service.load().unwrap();

        assert_eq!(loaded.model.provider, init_config.model.provider);
        assert_eq!(loaded.model.name, init_config.model.name);
    }

    #[test]
    fn load_without_init_uses_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        let config = service.load().unwrap();
        assert_eq!(config.model.provider, "local");
        assert_eq!(config.model.name, "BAAI/bge-small-en-v1.5");
    }

    #[test]
    fn update_model_persists() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        service
            .update_model("openai".to_string(), "text-embedding-3-small".to_string())
            .unwrap();

        let loaded = service.load().unwrap();
        assert_eq!(loaded.model.provider, "openai");
        assert_eq!(loaded.model.name, "text-embedding-3-small");
    }

    #[test]
    fn exists_returns_false_before_init() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        assert!(!service.exists());
    }

    #[test]
    fn exists_returns_true_after_init() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();
        assert!(service.exists());
    }

    #[test]
    fn resolve_storage_uri_defaults_to_local() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let uri = service.resolve_storage_uri(temp_dir.path()).unwrap();
        let expected = temp_dir
            .path()
            .join(".dna")
            .join("db")
            .join("artifacts.lance")
            .to_string_lossy()
            .to_string();
        assert_eq!(uri, expected);
    }

    #[test]
    fn resolve_storage_uri_preserves_s3_uri() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        let mut config = ProjectConfig::default();
        config.storage.uri = Some("s3://my-bucket/dna/artifacts.lance".to_string());
        service.save(&config).unwrap();

        let uri = service.resolve_storage_uri(temp_dir.path()).unwrap();
        assert_eq!(uri, "s3://my-bucket/dna/artifacts.lance");
    }

    #[test]
    fn resolve_storage_uri_resolves_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        let mut config = ProjectConfig::default();
        config.storage.uri = Some("custom/path.lance".to_string());
        service.save(&config).unwrap();

        let uri = service.resolve_storage_uri(temp_dir.path()).unwrap();
        let expected = temp_dir
            .path()
            .join("custom/path.lance")
            .to_string_lossy()
            .to_string();
        assert_eq!(uri, expected);
    }
}
