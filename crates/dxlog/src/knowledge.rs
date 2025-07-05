use anyhow::Result;
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use uuid::Uuid;

use crate::{
    config::Config,
    load_config,
    log_manager::LogManager,
    md_frontmatter::{extract_frontmatter, serialize_yaml_frontmatter},
    research_log::ResearchLog,
    utils::{self, Author, BaseLog},
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

pub struct KnowledgeManager {
    pub manager: LogManager<KnowledgeLog>,
}

impl KnowledgeManager {
    pub fn new(config: Config) -> Self {
        let search_dirs = vec![
            config.storage.active_dir.join("knowledge"),
            config.storage.knowledge_base_dir.join("knowledge"),
        ];

        Self {
            manager: LogManager::<KnowledgeLog>::new(config, search_dirs),
        }
    }

    pub fn create(&self, title: &str, tags: Option<Vec<String>>) -> Result<KnowledgeLog> {
        let author = utils::get_git_author()?;
        let knowledge = KnowledgeLog::new(title.to_string(), utils::normalize_tags(tags), author);

        let yaml = serialize_yaml_frontmatter(&knowledge)?;
        let template_content = utils::load_entry_content(&self.manager.config.templates.knowledge)?;

        let env = minijinja::Environment::new();
        let template = env.template_from_str(&template_content)?;
        let rendered = template.render(context! {
            research_log => yaml,
            title => knowledge.base.title,
        })?;

        self.manager.save_log(&knowledge, &rendered)?;
        Ok(knowledge)
    }

    pub fn update_status(&self, partial_id: &str, new_status: KnowledgeStatus) -> Result<()> {
        let (mut knowledge, file_path) = self.manager.find_log(partial_id)?;
        knowledge.update_status(new_status);
        self.manager.update_log(&mut knowledge, &file_path)
    }

    pub fn list(
        &self,
        status: Option<KnowledgeStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<KnowledgeLog>> {
        self.manager.list_logs(status, tags)
    }

    pub fn find(&self, partial_id: &str) -> Result<(KnowledgeLog, PathBuf)> {
        self.manager.find_log(partial_id)
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
    let manager = KnowledgeManager::new(config.clone());
    
    // Find the knowledge by partial ID
    let (mut knowledge, file_path) = manager.manager.find_log(partial_id)?;
    
    // Read the current file content
    let current_content = utils::load_entry_content(&file_path)?;
    
    // Create temporary file with current content
    let filename = format!("{}.md", knowledge.base.id);
    let temp_file = utils::create_temp_file_with_content(&current_content, &filename)?;
    
    // Get editor command and launch it
    let editor_command = utils::get_editor_command(&config)?;
    
    // Launch editor
    utils::launch_editor(&editor_command, &temp_file)?;
    
    // Read back the modified content
    let modified_content = utils::read_temp_file(&temp_file)?;
    
    // Parse the modified frontmatter to validate structure
    let (updated_knowledge, _): (KnowledgeLog, String) = 
        extract_frontmatter(&modified_content)?;
    
    // Validate that critical fields haven't been corrupted
    if updated_knowledge.base.id != knowledge.base.id {
        return Err(anyhow::anyhow!("ID cannot be modified"));
    }
    if updated_knowledge.base.date != knowledge.base.date {
        return Err(anyhow::anyhow!("Date cannot be modified"));
    }
    if updated_knowledge.base.created_by.name != knowledge.base.created_by.name 
        || updated_knowledge.base.created_by.email != knowledge.base.created_by.email {
        return Err(anyhow::anyhow!("Author cannot be modified"));
    }
    
    // Update the knowledge with the new data
    knowledge = updated_knowledge;
    
    // Use LogManager to update the log (handles file moves if status changed)
    manager.manager.update_log(&mut knowledge, &file_path)?;
    
    // Cleanup temp file
    utils::cleanup_temp_file(&temp_file)?;
    
    Ok(())
}
