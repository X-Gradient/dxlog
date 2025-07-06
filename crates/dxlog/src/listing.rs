use crate::{load_config, utils::load_entry_content, HypothesisLog, LiteratureLog, KnowledgeLog};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::{fs, path::Path};

#[derive(Debug)]
pub struct LogEntry {
    pub path: std::path::PathBuf,
    pub entry_type: String,
    pub title: String,
    pub id: String,
    pub status: String,
    pub tags: Vec<String>,
}

pub fn list_latest_active_logs(limit: usize) -> Result<()> {
    let config = load_config()?;
    let research_logs_dir = config.storage.active_dir.clone();
    
    if !research_logs_dir.exists() {
        println!("No research-logs directory found. Run 'dxlog init' to initialize a repository.");
        return Ok(());
    }
    
    let mut entries = Vec::new();
    
    // Collect all .md files from research-logs directory
    collect_md_files(&research_logs_dir, &mut entries)?;
    
    // Sort by modification time (newest first)
    entries.sort_by(|a, b| {
        let a_meta = fs::metadata(&a.path).unwrap_or_else(|_| panic!("Failed to get metadata for {}", a.path.display()));
        let b_meta = fs::metadata(&b.path).unwrap_or_else(|_| panic!("Failed to get metadata for {}", b.path.display()));
        b_meta.modified().unwrap().cmp(&a_meta.modified().unwrap())
    });
    
    // Display entries
    println!("{:<15} {:<25} {:<15} {:<12} {:<20} TAGS", "TYPE", "TITLE", "ID", "STATUS", "MODIFIED");
    println!("{}", "-".repeat(100));
    
    for (i, entry) in entries.iter().enumerate() {
        if i >= limit {
            break;
        }
        
        let meta = fs::metadata(&entry.path)?;
        let modified = meta.modified()?;
        let modified_dt: DateTime<Utc> = modified.into();
        let modified_str = modified_dt.format("%Y-%m-%d %H:%M").to_string();
        
        let short_id = if entry.id.len() > 12 {
            format!("{}...", &entry.id[..9])
        } else {
            entry.id.clone()
        };
        
        let short_title = if entry.title.len() > 25 {
            format!("{}...", &entry.title[..22])
        } else {
            entry.title.clone()
        };
        
        let tags = entry.tags.join(", ");
        
        println!(
            "{:<15} {:<25} {:<15} {:<12} {:<20} {}",
            entry.entry_type,
            short_title,
            short_id,
            entry.status,
            modified_str,
            tags
        );
    }
    
    if entries.len() > limit {
        println!("\n... and {} more entries (use --limit to see more)", entries.len() - limit);
    }
    
    Ok(())
}

fn collect_md_files(dir: &Path, entries: &mut Vec<LogEntry>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively check subdirectories
            collect_md_files(&path, entries)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            // Try to parse as different research log types
            let content = load_entry_content(&path)?;
            
            // Try to determine the type and extract basic info
            if let Ok(entry_info) = extract_log_info(&content, &path) {
                // Only include active entries based on type
                let should_include = match entry_info.entry_type.as_str() {
                    "Hypothesis" => entry_info.status == "active" || entry_info.status == "suspended",
                    "Literature" => entry_info.status == "in_progress",
                    "Knowledge" => entry_info.status == "draft",
                    _ => false,
                };
                
                if should_include {
                    entries.push(entry_info);
                }
            }
        }
    }
    
    Ok(())
}

fn extract_log_info(content: &str, path: &Path) -> Result<LogEntry> {
    use crate::md_frontmatter::extract_frontmatter;
    
    // Try to parse as different types
    if let Ok((hypothesis, _)) = extract_frontmatter::<HypothesisLog>(content) {
        return Ok(LogEntry {
            path: path.to_path_buf(),
            entry_type: "Hypothesis".to_string(),
            title: hypothesis.base.title.clone(),
            id: hypothesis.base.id.to_string(),
            status: hypothesis.status.to_string(),
            tags: hypothesis.base.tags.iter().cloned().collect(),
        });
    }
    
    if let Ok((literature, _)) = extract_frontmatter::<LiteratureLog>(content) {
        return Ok(LogEntry {
            path: path.to_path_buf(),
            entry_type: "Literature".to_string(),
            title: literature.base.title.clone(),
            id: literature.base.id.to_string(),
            status: literature.status.to_string(),
            tags: literature.base.tags.iter().cloned().collect(),
        });
    }
    
    if let Ok((knowledge, _)) = extract_frontmatter::<KnowledgeLog>(content) {
        return Ok(LogEntry {
            path: path.to_path_buf(),
            entry_type: "Knowledge".to_string(),
            title: knowledge.base.title.clone(),
            id: knowledge.base.id.to_string(),
            status: knowledge.status.to_string(),
            tags: knowledge.base.tags.iter().cloned().collect(),
        });
    }
    
    Err(anyhow::anyhow!("Could not parse log file: {}", path.display()))
}