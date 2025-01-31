use anyhow::Result;
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use uuid::Uuid;

use crate::{
    load_config,
    log_manager::LogManager,
    md_frontmatter::{extract_frontmatter, serialize_yaml_frontmatter},
    research_log::ResearchLog,
    utils::{self, generate_filename, Author, BaseLog},
    Config,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum HypothesisStatus {
    Active,
    Proven,
    Disproven,
    Inconclusive,
    Suspended,
    Abandoned,
}

impl ToString for HypothesisStatus {
    fn to_string(&self) -> String {
        match self {
            HypothesisStatus::Active => "active",
            HypothesisStatus::Proven => "proven",
            HypothesisStatus::Disproven => "disproven",
            HypothesisStatus::Inconclusive => "inconclusive",
            HypothesisStatus::Suspended => "suspended",
            HypothesisStatus::Abandoned => "abandoned",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HypothesisLog {
    #[serde(flatten)]
    pub base: BaseLog,
    pub status: HypothesisStatus,
}

impl ResearchLog for HypothesisLog {
    type Status = HypothesisStatus;

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
            status: HypothesisStatus::Active,
        }
    }

    fn update_status(&mut self, new_status: Self::Status) {
        self.status = new_status;
    }

    fn get_target_path(&self, config: &Config, current_path: &PathBuf) -> Result<PathBuf> {
        let filename = current_path.file_name().unwrap();
        match self.status {
            HypothesisStatus::Abandoned => Ok(config.storage.archive_dir.join(filename)),
            HypothesisStatus::Proven
            | HypothesisStatus::Disproven
            | HypothesisStatus::Inconclusive => Ok(config
                .storage
                .knowledge_base_dir
                .join("hypotheses")
                .join(filename)),
            _ => Ok(config.storage.active_dir.join(filename)),
        }
    }
}

pub struct HypothesisManager {
    pub manager: LogManager<HypothesisLog>,
}

impl HypothesisManager {
    pub fn new(config: Config) -> Self {
        let search_dirs = vec![
            config.storage.active_dir.clone(),
            config.storage.knowledge_base_dir.join("hypotheses"),
        ];
        Self {
            manager: LogManager::<HypothesisLog>::new(config, search_dirs),
        }
    }

    pub fn create(&self, title: &str, tags: Option<Vec<String>>) -> Result<HypothesisLog> {
        let author = utils::get_git_author()?;
        let hypothesis = HypothesisLog::new(title.to_string(), utils::normalize_tags(tags), author);

        let yaml = serialize_yaml_frontmatter(&hypothesis)?;
        let template_path = self.manager.config.templates.hypothesis.clone();
        let template_content = utils::load_entry_content(&template_path)?;

        let env = minijinja::Environment::new();
        let template = env.template_from_str(&template_content)?;
        let rendered = template.render(context! {
            research_log => yaml,
            title => hypothesis.base.title,
        })?;

        self.manager.save_log(&hypothesis, &rendered)?;
        Ok(hypothesis)
    }

    pub fn update_status(&self, partial_id: &str, new_status: HypothesisStatus) -> Result<()> {
        let (mut hypothesis, file_path): (HypothesisLog, PathBuf) =
            self.manager.find_log(partial_id)?;
        hypothesis.update_status(new_status);
        self.manager.update_log(&mut hypothesis, &file_path)
    }

    pub fn list(
        &self,
        status: Option<HypothesisStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<HypothesisLog>> {
        self.manager.list_logs(status, tags)
    }

    pub fn find(&self, partial_id: &str) -> Result<(HypothesisLog, PathBuf)> {
        self.manager.find_log(partial_id)
    }
}

pub fn create_hypothesis(title: &str, tags: Option<Vec<String>>) -> Result<HypothesisLog> {
    let config = load_config()?;
    let manager = HypothesisManager::new(config);
    manager.create(title, tags)
}

pub fn update_hypothesis_status(partial_id: &str, new_status: HypothesisStatus) -> Result<()> {
    let config = load_config()?;
    let manager = HypothesisManager::new(config);
    manager.update_status(partial_id, new_status)
}

pub fn list_hypotheses(
    status: Option<HypothesisStatus>,
    tags: Option<Vec<String>>,
) -> Result<Vec<HypothesisLog>> {
    let config = load_config()?;
    let manager = HypothesisManager::new(config);
    manager.list(status, tags)
}

pub fn _create_hypothesis(title: &str, tags: Option<Vec<String>>) -> Result<HypothesisLog> {
    let config = load_config()?;
    let author = utils::get_git_author()?;

    let hypothesis = HypothesisLog::new(
        title.to_string(),
        utils::normalize_tags(tags),
        author.clone(),
    );

    let yaml = serialize_yaml_frontmatter(&hypothesis)?;
    let template_path = config.templates.hypothesis;
    let template_content = utils::load_entry_content(&template_path)?;

    let env = minijinja::Environment::new();
    let template = env.template_from_str(&template_content)?;
    let rendered = template.render(context! {
        research_log => yaml,
        title => hypothesis.base.title,
    })?;

    let file_name = generate_filename(&hypothesis.base.title, &hypothesis.base.date);
    let file_path = config.storage.active_dir.join(&file_name);
    utils::save_entry_content(&file_path, &rendered)?;

    Ok(hypothesis)
}

pub fn _list_hypotheses(
    status: Option<HypothesisStatus>,
    tags: Option<Vec<String>>,
) -> Result<Vec<HypothesisLog>> {
    let config = load_config()?;
    let filter_tags = utils::normalize_tags(tags);
    let mut hypotheses = Vec::new();

    let search_dirs = [
        config.storage.active_dir.clone(),
        config.storage.knowledge_base_dir.join("hypotheses"),
    ];

    for dir in search_dirs {
        let hypothesis_files = utils::list_entries(&dir, "md")?;
        for file_path in hypothesis_files {
            let content = utils::load_entry_content(&file_path)?;
            let (hypothesis, _) = extract_frontmatter::<HypothesisLog>(&content)?;

            if let Some(target_status) = &status {
                if &hypothesis.status != target_status {
                    continue;
                }
            }

            if !filter_tags.is_empty() && !filter_tags.is_subset(&hypothesis.base.tags) {
                continue;
            }

            hypotheses.push(hypothesis);
        }
    }

    Ok(hypotheses)
}

pub fn _find_hypothesis_file(config: &Config, partial_id: &str) -> Result<PathBuf> {
    let search_dirs = [
        config.storage.active_dir.clone(),
        config.storage.knowledge_base_dir.join("hypotheses"),
    ];
    let mut matches = Vec::new();

    for dir in &search_dirs {
        let files = utils::list_entries(dir, "md")?;
        for file_path in files {
            let content = utils::load_entry_content(&file_path)?;
            let (hypothesis, _) = extract_frontmatter::<HypothesisLog>(&content)?;
            if hypothesis.base.id.to_string().starts_with(partial_id) {
                matches.push(file_path.clone());
            }
        }
    }

    match matches.len() {
        0 => anyhow::bail!("No hypothesis found with ID starting with '{}'", partial_id),
        1 => Ok(matches[0].clone()),
        _ => anyhow::bail!(
            "Multiple hypotheses found with ID starting with '{}'. Please provide more characters.",
            partial_id
        ),
    }
}
