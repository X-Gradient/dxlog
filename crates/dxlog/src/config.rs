use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub date_format: String,
    pub templates: TemplateConfig,
    pub storage: StorageConfig,
    pub stale_days: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub active_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub knowledge_base_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateConfig {
    pub hypothesis: PathBuf,
    pub literature: PathBuf,
    pub knowledge: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            date_format: "%Y-%m-%d".to_string(),
            templates: TemplateConfig {
                hypothesis: "templates/hypothesis.jinja".into(),
                literature: "templates/literature.jinja".into(),
                knowledge: "templates/knowledge.jinja".into(),
            },
            storage: StorageConfig {
                active_dir: "research-logs".into(),
                archive_dir: "archived".into(),
                knowledge_base_dir: "knowledge-base".into(),
            },
            stale_days: 14,
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = std::path::Path::new(".rlog.toml");
    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
