mod arxiv_scraper;
mod github_scraper;

pub use arxiv_scraper::{fetch_arxiv_metadata, ArxivMetadata};
pub use github_scraper::{fetch_github_metadata, GitHubRepo};
