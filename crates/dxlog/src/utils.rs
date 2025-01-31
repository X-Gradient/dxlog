use anyhow::{Context, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseLog {
    pub id: Uuid,
    pub date: String,
    pub title: String,
    pub tags: HashSet<String>,
    pub created_by: Author,
    pub references: HashSet<Uuid>,
}

pub struct StatusChange {
    pub from: String,
    pub to: String,
    pub reason: String,
}

pub fn generate_filename(title: &str, date: &str) -> String {
    // Sanitize title: lowercase, replace spaces with hyphens, remove special chars
    let safe_title = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-");

    format!("{}-{}.md", date, safe_title)
}

pub fn get_git_author() -> Result<Author> {
    let repo = Repository::open_from_env()
        .context("Failed to open git repository. Make sure you're in a git repository")?;

    let config = repo.config()?;

    let name = config
        .get_string("user.name")
        .context("Git user.name not configured")?;
    let email = config
        .get_string("user.email")
        .context("Git user.email not configured")?;

    Ok(Author { name, email })
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn normalize_tags(tags: Option<Vec<String>>) -> HashSet<String> {
    tags.map(|t| t.into_iter().collect()).unwrap_or_default()
}

pub fn load_entry_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

pub fn save_entry_content(path: &Path, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }

    if path.exists() {
        return Err(anyhow::anyhow!("File already exists: {}", path.display()));
    }

    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

pub fn list_entries(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    if !dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some(extension) {
            entries.push(path);
        }
    }

    Ok(entries)
}

pub fn commit_changes(paths: &[PathBuf], message: &str) -> Result<()> {
    let repo = Repository::open_from_env()?;
    let mut index = repo.index()?;

    for path in paths {
        let relative_path = path.strip_prefix(repo.workdir().unwrap())?;
        index.add_path(relative_path)?;
    }

    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let signature = repo.signature()?;
    let parent = repo.head()?.peel_to_commit()?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent],
    )?;

    Ok(())
}

pub fn detect_cycles(references: &HashSet<Uuid>, new_ref: Uuid, logs: &[BaseLog]) -> bool {
    let mut visited = HashSet::new();
    let mut stack = vec![new_ref];

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            return true;
        }
        if let Some(log) = logs.iter().find(|l| l.id == current) {
            stack.extend(log.references.iter());
        }
    }
    false
}

pub fn add_reference(log: &mut BaseLog, ref_id: Uuid, all_logs: &[BaseLog]) -> Result<()> {
    if detect_cycles(&log.references, ref_id, all_logs) {
        return Err(anyhow::anyhow!(
            "Adding this reference would create a cycle"
        ));
    }
    log.references.insert(ref_id);
    Ok(())
}

pub fn remove_reference(log: &mut BaseLog, ref_id: &Uuid) {
    log.references.remove(ref_id);
}
