use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub default_branch: String,
    pub topics: Vec<String>,
}

fn extract_repo_info(url: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = url.trim_end_matches(".git").split('/').collect();

    match &parts[..] {
        [.., owner, repo] => Some((owner.to_string(), repo.to_string())),
        _ => None,
    }
}

pub fn fetch_github_metadata(url: &str) -> Result<GitHubRepo> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let (owner, repo) = extract_repo_info(url).context("Invalid GitHub URL format")?;

    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let response = client
        .get(api_url)
        .header("User-Agent", "dxlog/0.1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()?;

    let repo_data: GitHubRepo = response.json()?;

    Ok(repo_data)
}
