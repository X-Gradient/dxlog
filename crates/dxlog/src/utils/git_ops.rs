use super::Author;
use anyhow::{Context, Result};
use git2::Repository;
use std::path::PathBuf;

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