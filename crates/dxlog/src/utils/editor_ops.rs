use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_editor_command(config: &crate::Config) -> Result<String> {
    // Priority: DXLOG_EDITOR env var -> config file -> EDITOR env var -> default
    if let Ok(editor) = env::var("DXLOG_EDITOR") {
        return Ok(editor);
    }
    
    if let Some(ref editor) = config.editor {
        return Ok(editor.clone());
    }
    
    if let Ok(editor) = env::var("EDITOR") {
        return Ok(editor);
    }
    
    // Default fallback
    Ok("nano".to_string())
}

pub fn create_temp_file_with_content(content: &str, filename: &str) -> Result<PathBuf> {
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(format!("dxlog-edit-{}", filename));
    fs::write(&temp_file, content)?;
    Ok(temp_file)
}

pub fn read_temp_file(temp_file: &Path) -> Result<String> {
    fs::read_to_string(temp_file)
        .with_context(|| format!("Failed to read temporary file: {}", temp_file.display()))
}

pub fn cleanup_temp_file(temp_file: &Path) -> Result<()> {
    if temp_file.exists() {
        fs::remove_file(temp_file)
            .with_context(|| format!("Failed to cleanup temporary file: {}", temp_file.display()))?;
    }
    Ok(())
}

pub fn launch_editor(editor_command: &str, file_path: &Path) -> Result<()> {
    let parts: Vec<&str> = editor_command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty editor command"));
    }

    let mut command = Command::new(parts[0]);
    
    // Add any additional arguments
    if parts.len() > 1 {
        command.args(&parts[1..]);
    }
    
    // Add the file path
    command.arg(file_path);

    let status = command.status()
        .with_context(|| format!("Failed to execute editor command: {}", editor_command))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Editor exited with non-zero status: {:?}", 
            status.code()
        ));
    }

    Ok(())
}