// crates/dxlog/src/generic_manager.rs
use crate::{
    config::{Config, TemplateConfig},
    log_manager::LogManager,
    md_frontmatter::serialize_yaml_frontmatter,
    research_log::ResearchLog,
    utils::{self, get_editor_command, launch_editor, BaseLog},
};
use anyhow::Result;
use minijinja::context;
use std::path::PathBuf;
use uuid::Uuid;

/// Generic manager for all research log types
/// Eliminates code duplication across HypothesisManager, LiteratureManager, and KnowledgeManager
pub struct GenericManager<T: ResearchLog> {
    pub config: Config,
    pub log_manager: LogManager<T>,
    template_path: PathBuf,
}

impl<T: ResearchLog> GenericManager<T> {
    pub fn new(config: Config) -> Self {
        let search_dirs = Self::build_search_dirs(&config);
        let template_path = Self::get_template_path(&config);
        
        Self {
            log_manager: LogManager::new(config.clone(), search_dirs),
            config,
            template_path,
        }
    }

    fn build_search_dirs(config: &Config) -> Vec<PathBuf> {
        vec![
            config.storage.active_dir.join(T::subdirectory_name()),
            config.storage.knowledge_base_dir.join(T::subdirectory_name()),
        ]
    }

    fn get_template_path(config: &Config) -> PathBuf {
        // This is a bit of a workaround since we can't have trait-specific associated functions
        // that return different template paths. We'll match on the subdirectory name.
        match T::subdirectory_name() {
            "hypotheses" => config.templates.hypothesis.clone(),
            "literature" => config.templates.literature.clone(),
            "knowledge" => config.templates.knowledge.clone(),
            _ => config.templates.hypothesis.clone(), // fallback
        }
    }

    pub fn find(&self, partial_id: &str) -> Result<(T, PathBuf)> {
        self.log_manager.find_log(partial_id)
    }

    pub fn list(&self, status: Option<T::Status>, tags: Option<Vec<String>>) -> Result<Vec<T>> {
        self.log_manager.list_logs(status, tags)
    }

    pub fn create(&self, title: &str, tags: Option<Vec<String>>) -> Result<T> {
        let author = utils::get_git_author()?;
        let normalized_tags = utils::normalize_tags(tags);

        let log = T::new(title.to_string(), normalized_tags, author);

        // Render template
        let content = self.render_template(&log)?;

        // Save to file
        let file_path = self.log_manager.save_log(&log, &content)?;
        
        // Commit changes if git is enabled
        if self.config.git.enabled && self.config.git.auto_commit {
            let commit_message = format!("Add new {}: {}", T::subdirectory_name(), title);
            utils::commit_changes(&[file_path], &commit_message)?;
        }

        Ok(log)
    }

    pub fn update_status(&self, partial_id: &str, new_status: T::Status) -> Result<()> {
        let (mut log, file_path) = self.find(partial_id)?;
        log.update_status(new_status);
        self.log_manager.update_log(&mut log, &file_path)?;

        // Commit changes if git is enabled
        if self.config.git.enabled && self.config.git.auto_commit {
            let commit_message = format!(
                "Update {} status: {} -> {}",
                T::subdirectory_name(),
                log.base().title,
                log.status().to_string()
            );
            utils::commit_changes(&[file_path], &commit_message)?;
        }

        Ok(())
    }

    pub fn edit(&self, partial_id: &str) -> Result<()> {
        let (mut log, file_path) = self.find(partial_id)?;
        
        // Get editor command
        let editor_command = get_editor_command(&self.config)?;
        
        // Create temporary file with current content
        let current_content = utils::load_entry_content(&file_path)?;
        let temp_file = utils::create_temp_file_with_content(
            &current_content,
            &format!("{}-edit.md", log.base().id)
        )?;
        
        // Launch editor
        launch_editor(&editor_command, &temp_file)?;
        
        // Read back the modified content
        let modified_content = utils::read_temp_file(&temp_file)?;
        
        // Parse the modified frontmatter to validate structure
        let (updated_log, _): (T, String) = 
            crate::md_frontmatter::extract_frontmatter(&modified_content)?;
        
        // Validate that critical fields haven't been corrupted
        self.validate_edit_changes(&log, &updated_log)?;
        
        // Update the log with the new data
        log = updated_log;
        
        // Use LogManager to update the log (handles file moves if status changed)
        self.log_manager.update_log(&mut log, &file_path)?;
        
        // Cleanup temp file
        utils::cleanup_temp_file(&temp_file)?;
        
        Ok(())
    }

    fn validate_edit_changes(&self, original: &T, updated: &T) -> Result<()> {
        if original.base().id != updated.base().id {
            return Err(anyhow::anyhow!("ID cannot be modified"));
        }
        if original.base().date != updated.base().date {
            return Err(anyhow::anyhow!("Date cannot be modified"));
        }
        if original.base().created_by.name != updated.base().created_by.name 
            || original.base().created_by.email != updated.base().created_by.email {
            return Err(anyhow::anyhow!("Author cannot be modified"));
        }
        Ok(())
    }

    fn render_template(&self, log: &T) -> Result<String> {
        let yaml = serialize_yaml_frontmatter(log)?;
        let template_content = utils::load_entry_content(&self.template_path)?;

        let env = minijinja::Environment::new();
        let template = env.template_from_str(&template_content)?;
        let rendered = template.render(context! {
            research_log => yaml,
            title => log.base().title,
        })?;

        Ok(rendered)
    }

    /// Get all logs as BaseLog for reference operations
    pub fn get_all_base_logs(&self) -> Result<Vec<BaseLog>> {
        let logs = self.list(None, None)?;
        Ok(logs.into_iter().map(|log| log.base().clone()).collect())
    }

    /// Add a reference to a log
    pub fn add_reference(&self, source_id: &str, target_uuid: Uuid) -> Result<()> {
        let (mut log, file_path) = self.find(source_id)?;
        log.base_mut().references.insert(target_uuid);
        self.log_manager.update_log(&mut log, &file_path)
    }

    /// Remove a reference from a log
    pub fn remove_reference(&self, source_id: &str, target_uuid: Uuid) -> Result<()> {
        let (mut log, file_path) = self.find(source_id)?;
        log.base_mut().references.remove(&target_uuid);
        self.log_manager.update_log(&mut log, &file_path)
    }
}

/// Helper trait to get template path for each type
pub trait TemplatePathProvider {
    fn template_path(config: &TemplateConfig) -> &PathBuf;
}

// We'll need to implement this for each research log type to avoid the match statement
impl<T: ResearchLog + TemplatePathProvider> GenericManager<T> {
    pub fn new_with_template_provider(config: Config) -> Self {
        let search_dirs = Self::build_search_dirs(&config);
        let template_path = T::template_path(&config.templates).clone();
        
        Self {
            log_manager: LogManager::new(config.clone(), search_dirs),
            config,
            template_path,
        }
    }
}