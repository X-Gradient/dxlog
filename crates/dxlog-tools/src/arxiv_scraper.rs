use anyhow::{Context, Result};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(Debug)]
pub struct ArxivMetadata {
    pub title: String,
    pub abstract_text: String,
    pub doi: String,
}

fn extract_arxiv_id(url: &str) -> Option<String> {
    url.split('/')
        .last()
        .map(|s| s.trim_start_matches("abs/").to_string())
}

pub fn fetch_arxiv_metadata(url: &str) -> Result<ArxivMetadata> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    // Convert URL to abs format if needed
    let arxiv_id = extract_arxiv_id(url).context("Failed to extract arXiv ID")?;
    let abs_url = format!("https://arxiv.org/abs/{}", arxiv_id);

    let response = client
        .get(abs_url)
        .header("User-Agent", "dxlog/0.1.0")
        .send()?
        .text()?;

    let document = Html::parse_document(&response);

    // Extract abstract
    let abstract_selector = Selector::parse("blockquote.abstract").unwrap();
    let abstract_text = document
        .select(&abstract_selector)
        .next()
        .context("Abstract not found")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let title_selector = Selector::parse("h1.title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .context("Title not found")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // Extract DOI - it's in a link with class "arxiv-doi"
    let doi_selector = Selector::parse("#arxiv-doi-link").unwrap();
    let doi = document
        .select(&doi_selector)
        .next()
        .and_then(|el| {
            el.attr("href")
                .map(|href| href.trim_start_matches("https://doi.org/").to_string())
        })
        .expect("DOI not found");

    Ok(ArxivMetadata {
        title,
        abstract_text,
        doi,
    })
}
