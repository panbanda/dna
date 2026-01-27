use super::types::{ModelConfig, ProjectConfig};
use anyhow::{Context, Result};
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

    /// Load configuration from file
    pub fn load(&self) -> Result<ProjectConfig> {
        let content =
            std::fs::read_to_string(&self.config_path).context("Failed to read config file")?;
        let config: ProjectConfig =
            toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config: &ProjectConfig) -> Result<()> {
        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        std::fs::write(&self.config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    /// Update model configuration
    pub fn update_model(&self, provider: String, name: String) -> Result<ModelConfig> {
        let mut config = self.load()?;
        config.model.provider = provider;
        config.model.name = name;
        self.save(&config)?;
        Ok(config.model)
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Result<String> {
        let config = self.load()?;
        match key {
            "model.provider" => Ok(config.model.provider),
            "model.name" => Ok(config.model.name),
            "model.api_key_env" => Ok(config.model.api_key_env.unwrap_or_default()),
            "model.base_url" => Ok(config.model.base_url.unwrap_or_default()),
            _ => Err(anyhow::anyhow!("Unknown config key: {}", key)),
        }
    }

    /// Set a configuration value
    pub fn set(&self, key: &str, value: String) -> Result<()> {
        let mut config = self.load()?;
        match key {
            "model.provider" => config.model.provider = value,
            "model.name" => config.model.name = value,
            "model.api_key_env" => config.model.api_key_env = Some(value),
            "model.base_url" => config.model.base_url = Some(value),
            _ => return Err(anyhow::anyhow!("Unknown config key: {}", key)),
        }
        self.save(&config)?;
        Ok(())
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
    fn load_without_init_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        let result = service.load();
        assert!(result.is_err());
    }

    #[test]
    fn update_model_persists() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let model = service
            .update_model("openai".to_string(), "text-embedding-3-small".to_string())
            .unwrap();
        assert_eq!(model.provider, "openai");
        assert_eq!(model.name, "text-embedding-3-small");

        let loaded = service.load().unwrap();
        assert_eq!(loaded.model.provider, "openai");
        assert_eq!(loaded.model.name, "text-embedding-3-small");
    }

    #[test]
    fn get_model_provider() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let value = service.get("model.provider").unwrap();
        assert_eq!(value, "local");
    }

    #[test]
    fn get_model_name() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let value = service.get("model.name").unwrap();
        assert_eq!(value, "BAAI/bge-small-en-v1.5");
    }

    #[test]
    fn get_unknown_key_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let result = service.get("unknown.key");
        assert!(result.is_err());
    }

    #[test]
    fn set_model_provider() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        service.set("model.provider", "ollama".to_string()).unwrap();
        let value = service.get("model.provider").unwrap();
        assert_eq!(value, "ollama");
    }

    #[test]
    fn set_model_name() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        service
            .set("model.name", "custom-model".to_string())
            .unwrap();
        let value = service.get("model.name").unwrap();
        assert_eq!(value, "custom-model");
    }

    #[test]
    fn set_model_api_key_env() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        service
            .set("model.api_key_env", "MY_API_KEY".to_string())
            .unwrap();
        let value = service.get("model.api_key_env").unwrap();
        assert_eq!(value, "MY_API_KEY");
    }

    #[test]
    fn set_model_base_url() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        service
            .set("model.base_url", "http://localhost:8080".to_string())
            .unwrap();
        let value = service.get("model.base_url").unwrap();
        assert_eq!(value, "http://localhost:8080");
    }

    #[test]
    fn set_unknown_key_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigService::new(temp_dir.path());
        service.init().unwrap();

        let result = service.set("unknown.key", "value".to_string());
        assert!(result.is_err());
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
}
