use anyhow::Result;
use chrono::Local;
use dxlog_tools::{fetch_arxiv_metadata, fetch_github_metadata};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::{load_config, Config};
use crate::log_manager::LogManager;
use crate::md_frontmatter::{extract_frontmatter, serialize_yaml_frontmatter};
use crate::research_log::ResearchLog;
use crate::utils::{self, Author, BaseLog};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum LiteratureStatus {
    InProgress,
    Completed,
    Archived,
}

impl ToString for LiteratureStatus {
    fn to_string(&self) -> String {
        match self {
            LiteratureStatus::InProgress => "in_progress",
            LiteratureStatus::Completed => "completed",
            LiteratureStatus::Archived => "archived",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Source {
    pub doi: Option<String>,
    pub arxiv_url: Option<String>,
    pub pdf_url: Option<String>,
    pub repository_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiteratureLog {
    #[serde(flatten)]
    pub base: BaseLog,
    pub status: LiteratureStatus,
    pub source: Source,
    #[serde(skip)]
    pub abstract_text: Option<String>,
    #[serde(skip)]
    pub repository_description: Option<String>,
}

impl ResearchLog for LiteratureLog {
    type Status = LiteratureStatus;

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
        let now = Local::now();
        Self {
            base: BaseLog {
                id: Uuid::new_v4(),
                date: now.format("%Y-%m-%d").to_string(),
                title,
                tags,
                created_by: author,
                references: HashSet::new(),
            },
            status: LiteratureStatus::InProgress,
            source: Source::default(),
            abstract_text: None,
            repository_description: None,
        }
    }

    fn update_status(&mut self, new_status: Self::Status) {
        self.status = new_status;
    }

    fn get_target_path(&self, config: &Config, current_path: &PathBuf) -> Result<PathBuf> {
        let filename = current_path.file_name().unwrap();
        let lit_path = |base: PathBuf| base.join("literature").join(filename);

        match self.status {
            LiteratureStatus::Archived => Ok(lit_path(config.storage.archive_dir.clone())),
            LiteratureStatus::Completed => Ok(lit_path(config.storage.knowledge_base_dir.clone())),
            _ => Ok(lit_path(config.storage.active_dir.clone())),
        }
    }
}

pub struct LiteratureManager {
    pub manager: LogManager<LiteratureLog>,
}

impl LiteratureManager {
    pub fn new(config: Config) -> Self {
        let search_dirs = vec![
            config.storage.active_dir.join("literature"),
            config.storage.knowledge_base_dir.join("literature"),
        ];
        Self {
            manager: LogManager::new(config, search_dirs),
        }
    }

    pub fn create(&self, url: &str, tags: Option<Vec<String>>) -> Result<LiteratureLog> {
        let author = utils::get_git_author()?;

        let source = if url.contains("arxiv.org") {
            Source {
                arxiv_url: Some(url.to_string()),
                ..Default::default()
            }
        } else if url.contains("github.com") {
            Source {
                repository_url: Some(url.to_string()),
                ..Default::default()
            }
        } else if url.starts_with("10.") {
            Source {
                doi: Some(url.to_string()),
                ..Default::default()
            }
        } else {
            return Err(anyhow::anyhow!("Unsupported URL format"));
        };

        let (title, abstract_text, repo_description) = fetch_metadata(&source)?;
        let mut literature = LiteratureLog::new(title, utils::normalize_tags(tags), author);
        literature.source = source;
        literature.abstract_text = abstract_text.clone();
        literature.repository_description = repo_description;

        let yaml = serialize_yaml_frontmatter(&literature)?;
        let template_content =
            utils::load_entry_content(&self.manager.config.templates.literature)?;

        let env = minijinja::Environment::new();
        let template = env.template_from_str(&template_content)?;
        let rendered = template.render(context! {
            research_log => yaml,
            title => literature.base.title,
            abstract_text => abstract_text,
        })?;

        self.manager.save_log(&literature, &rendered)?;
        Ok(literature)
    }

    pub fn update_status(&self, partial_id: &str, new_status: LiteratureStatus) -> Result<()> {
        let (mut literature, file_path) = self.manager.find_log(partial_id)?;
        literature.update_status(new_status);
        self.manager.update_log(&mut literature, &file_path)
    }

    pub fn delete(&self, partial_id: &str) -> Result<()> {
        let (_, file_path) = self.manager.find_log(partial_id)?;
        std::fs::remove_file(file_path)?;
        Ok(())
    }

    pub fn list(
        &self,
        status: Option<LiteratureStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<LiteratureLog>> {
        self.manager.list_logs(status, tags)
    }

    pub fn find(&self, partial_id: &str) -> Result<(LiteratureLog, PathBuf)> {
        self.manager.find_log(partial_id)
    }
}

pub fn create_literature(url: &str, tags: Option<Vec<String>>) -> Result<LiteratureLog> {
    let config = load_config()?;
    let manager = LiteratureManager::new(config);
    manager.create(url, tags)
}

pub fn update_literature_status(partial_id: &str, new_status: LiteratureStatus) -> Result<()> {
    let config = load_config()?;
    let manager = LiteratureManager::new(config);
    manager.update_status(partial_id, new_status)
}

pub fn delete_literature(partial_id: &str) -> Result<()> {
    let config = load_config()?;
    let manager = LiteratureManager::new(config);
    manager.delete(partial_id)
}

pub fn list_literature(
    status: Option<LiteratureStatus>,
    tags: Option<Vec<String>>,
) -> Result<Vec<LiteratureLog>> {
    let config = load_config()?;
    let manager = LiteratureManager::new(config);
    manager.list(status, tags)
}

pub fn _find_literature_file(config: &Config, partial_id: &str) -> Result<PathBuf> {
    let search_dirs = [
        config.storage.active_dir.join("literature"),
        config.storage.knowledge_base_dir.join("literature"),
    ];
    let mut matches = Vec::new();

    for dir in &search_dirs {
        let files = utils::list_entries(dir, "md")?;
        for file_path in files {
            let content = utils::load_entry_content(&file_path)?;
            let (literature, _) = extract_frontmatter::<LiteratureLog>(&content)?;
            if literature.base.id.to_string().starts_with(partial_id) {
                matches.push(file_path.clone());
            }
        }
    }

    match matches.len() {
        0 => anyhow::bail!("No literature found with ID starting with '{}'", partial_id),
        1 => Ok(matches[0].clone()),
        _ => anyhow::bail!(
            "Multiple literatures found with ID starting with '{}'. Please provide more characters.",
            partial_id
        ),
    }
}

pub fn fetch_metadata(source: &Source) -> Result<(String, Option<String>, Option<String>)> {
    let mut title = String::new();
    let mut abstract_text = None;
    let mut repo_description = None;

    if let Some(arxiv_url) = &source.arxiv_url {
        let metadata = fetch_arxiv_metadata(arxiv_url)?;
        title = metadata.title;
        abstract_text = Some(metadata.abstract_text);
    } else if let Some(repo_url) = &source.repository_url {
        if repo_url.contains("github.com") {
            let git_repo = fetch_github_metadata(repo_url)?;
            repo_description = git_repo.description;
            title = git_repo.name;
        }
    }

    Ok((title, abstract_text, repo_description))
}
