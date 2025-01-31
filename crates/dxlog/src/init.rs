use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn init_repository(base_path: &Path) -> Result<()> {
    let dirs = [
        "templates",
        "archived",
        "knowledge-base/literature",
        "knowledge-base/hypotheses",
        "research-logs",
    ];

    for dir in dirs.iter() {
        let path = base_path.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }

    let hypothesis_template = include_str!("templates/hypothesis.default.jinja");
    let literature_template = include_str!("templates/literature.default.jinja");
    let knowledge_template = include_str!("templates/knowledge.default.jinja");

    fs::write(
        base_path.join("templates/hypothesis.jinja"),
        hypothesis_template,
    )
    .with_context(|| "Failed to write hypothesis template")?;
    fs::write(
        base_path.join("templates/literature.jinja"),
        literature_template,
    )
    .with_context(|| "Failed to write literature template")?;

    fs::write(
        base_path.join("templates/knowledge.jinja"),
        knowledge_template,
    )
    .with_context(|| "Failed to write knowledge template")?;

    create_default_config(base_path)?;

    Ok(())
}

fn create_default_config(base_path: &Path) -> Result<()> {
    let config_content = include_str!("../../../dxlog.toml");
    let config_path = base_path.join("dxlog.toml");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
    Ok(())
}
