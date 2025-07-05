use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use uuid::Uuid;

use crate::{
    config::{Config, TemplateConfig},
    generic_manager::{GenericManager, TemplatePathProvider},
    load_config,
    research_log::ResearchLog,
    utils::{Author, BaseLog},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum KnowledgeStatus {
    Draft,
    Published,
    Archived,
}

impl ToString for KnowledgeStatus {
    fn to_string(&self) -> String {
        match self {
            KnowledgeStatus::Draft => "draft",
            KnowledgeStatus::Published => "published",
            KnowledgeStatus::Archived => "archived",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KnowledgeLog {
    #[serde(flatten)]
    pub base: BaseLog,
    pub status: KnowledgeStatus,
}

impl ResearchLog for KnowledgeLog {
    type Status = KnowledgeStatus;

    fn base(&self) -> &BaseLog {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseLog {
        &mut self.base
    }

    fn status(&self) -> &Self::Status {
        &self.status
    }

    fn status_mut(&mut self) -> &mut Self::Status {
        &mut self.status
    }

    fn new(title: String, tags: HashSet<String>, author: Author) -> Self {
        let now = chrono::Local::now();
        Self {
            base: BaseLog {
                id: Uuid::new_v4(),
                date: now.format("%Y-%m-%d").to_string(),
                title,
                tags,
                created_by: author,
                references: HashSet::new(),
            },
            status: KnowledgeStatus::Draft,
        }
    }

    fn update_status(&mut self, new_status: Self::Status) {
        self.status = new_status;
    }

    fn get_target_path(&self, config: &Config, current_path: &PathBuf) -> Result<PathBuf> {
        let filename = current_path.file_name().unwrap();
        match self.status {
            KnowledgeStatus::Archived => Ok(config.storage.archive_dir.join("knowledge").join(filename)),
            KnowledgeStatus::Published => Ok(config.storage.knowledge_base_dir.join("knowledge").join(filename)),
            _ => Ok(config.storage.active_dir.join("knowledge").join(filename)),
        }
    }

    fn subdirectory_name() -> &'static str {
        "knowledge"
    }

}

impl TemplatePathProvider for KnowledgeLog {
    fn template_path(config: &TemplateConfig) -> &PathBuf {
        &config.knowledge
    }
}

pub struct KnowledgeManager {
    pub manager: GenericManager<KnowledgeLog>,
}

impl KnowledgeManager {
    pub fn new(config: Config) -> Self {
        Self {
            manager: GenericManager::new_with_template_provider(config),
        }
    }

    pub fn create(&self, title: &str, tags: Option<Vec<String>>) -> Result<KnowledgeLog> {
        self.manager.create(title, tags)
    }

    pub fn update_status(&self, partial_id: &str, new_status: KnowledgeStatus) -> Result<()> {
        self.manager.update_status(partial_id, new_status)
    }

    pub fn list(
        &self,
        status: Option<KnowledgeStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<KnowledgeLog>> {
        self.manager.list(status, tags)
    }

    pub fn find(&self, partial_id: &str) -> Result<(KnowledgeLog, PathBuf)> {
        self.manager.find(partial_id)
    }

    pub fn edit(&self, partial_id: &str) -> Result<()> {
        self.manager.edit(partial_id)
    }
}

pub fn create_knowledge(title: &str, tags: Option<Vec<String>>) -> Result<KnowledgeLog> {
    let config = load_config()?;
    let manager = KnowledgeManager::new(config);
    manager.create(title, tags)
}

pub fn update_knowledge_status(partial_id: &str, new_status: KnowledgeStatus) -> Result<()> {
    let config = load_config()?;
    let manager = KnowledgeManager::new(config);
    manager.update_status(partial_id, new_status)
}

pub fn list_knowledge(
    status: Option<KnowledgeStatus>,
    tags: Option<Vec<String>>,
) -> Result<Vec<KnowledgeLog>> {
    let config = load_config()?;
    let manager = KnowledgeManager::new(config);
    manager.list(status, tags)
}

pub fn edit_knowledge(partial_id: &str) -> Result<()> {
    let config = load_config()?;
    let manager = KnowledgeManager::new(config);
    manager.edit(partial_id)
}
