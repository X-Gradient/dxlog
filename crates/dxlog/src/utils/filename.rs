use std::collections::HashSet;

pub fn generate_filename(title: &str, date: &str) -> String {
    // Sanitize title: lowercase, replace spaces with hyphens, remove special chars
    let safe_title = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-");

    format!("{}-{}.md", date, safe_title)
}

pub fn normalize_tags(tags: Option<Vec<String>>) -> HashSet<String> {
    tags.map(|t| t.into_iter().collect()).unwrap_or_default()
}