use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
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