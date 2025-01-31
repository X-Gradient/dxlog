use crate::{
    md_frontmatter::{extract_frontmatter, update_markdown_frontmatter},
    research_log::ResearchLog,
    utils::{self, load_entry_content, save_entry_content},
    Config,
};
use anyhow::Result;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub struct LogManager<T: ResearchLog> {
    pub(crate) config: Config,
    search_dirs: Vec<PathBuf>,
    phantom_data: PhantomData<T>,
}

impl<T: ResearchLog> LogManager<T> {
    pub fn new(config: Config, search_dirs: Vec<PathBuf>) -> Self {
        Self {
            config,
            search_dirs,
            phantom_data: PhantomData,
        }
    }

    pub fn find_log(&self, partial_id: &str) -> Result<(T, PathBuf)> {
        let mut matches = Vec::new();

        for dir in &self.search_dirs {
            let files = utils::list_entries(dir, "md")?;
            for file_path in files {
                let content = load_entry_content(&file_path)?;
                let (log, _) = extract_frontmatter::<T>(&content)?;
                if log.base().id.to_string().starts_with(partial_id) {
                    matches.push((log, file_path.clone()));
                }
            }
        }

        match matches.len() {
            0 => Err(anyhow::anyhow!(
                "No log found with ID starting with '{}'",
                partial_id
            )),
            1 => Ok(matches.remove(0)),
            _ => Err(anyhow::anyhow!(
                "Multiple logs found with ID starting with '{}'. Please provide more characters.",
                partial_id
            )),
        }
    }

    pub fn list_logs(
        &self,
        status: Option<T::Status>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<T>> {
        let filter_tags = utils::normalize_tags(tags);
        let mut logs = Vec::new();

        for dir in &self.search_dirs {
            let files = utils::list_entries(dir, "md")?;
            for file_path in files {
                let content = load_entry_content(&file_path)?;
                let (log, _) = extract_frontmatter::<T>(&content)?;

                if let Some(target_status) = &status {
                    if log.status().to_string() != target_status.to_string() {
                        continue;
                    }
                }

                if !filter_tags.is_empty() && !filter_tags.is_subset(&log.base().tags) {
                    continue;
                }

                logs.push(log);
            }
        }

        Ok(logs)
    }

    fn find_existing_log(&self, title: &str) -> Result<Option<(String, PathBuf)>> {
        for dir in &self.search_dirs {
            let files = utils::list_entries(dir, "md")?;
            for file_path in files {
                let content = load_entry_content(&file_path)?;
                let (log, _) = extract_frontmatter::<T>(&content)?;
                if log.base().title.to_lowercase() == title.to_lowercase() {
                    return Ok(Some((log.base().title.clone(), file_path)));
                }
            }
        }
        Ok(None)
    }

    pub fn save_log(&self, log: &T, content: &str) -> Result<PathBuf> {
        if let Some((existing_title, existing_path)) = self.find_existing_log(&log.base().title)? {
            return Err(anyhow::anyhow!(
                "A research log with title '{}' already exists at: {}",
                existing_title,
                existing_path.display()
            ));
        }
        let file_name = utils::generate_filename(&log.base().title, &log.base().date);
        let file_path = self.config.storage.active_dir.join(&file_name);
        save_entry_content(&file_path, content)?;
        Ok(file_path)
    }

    pub fn update_log(&self, log: &mut T, file_path: &Path) -> Result<()> {
        let content = load_entry_content(file_path)?;
        let (_, content) = extract_frontmatter::<T>(&content)?;
        let updated_content = update_markdown_frontmatter(log, &content)?;

        let new_path = log.get_target_path(&self.config, &file_path.to_path_buf())?;
        utils::ensure_directory(new_path.parent().unwrap())?;
        std::fs::rename(file_path, &new_path)?;
        std::fs::write(new_path, updated_content)?;
        Ok(())
    }
}
