use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

const YAML_SEPARATOR: &str = "---";

pub fn extract_frontmatter<T: DeserializeOwned>(content: &str) -> Result<(T, String)> {
    let parts: Vec<&str> = content.splitn(3, YAML_SEPARATOR).collect();

    match parts.as_slice() {
        ["", yaml, content] => {
            let frontmatter: T = serde_yaml::from_str(yaml.trim())?;
            Ok((frontmatter, content.trim().to_string()))
        }
        _ => Err(anyhow::anyhow!("Invalid markdown format")),
    }
}

pub fn serialize_yaml_frontmatter<T: Serialize>(data: &T) -> Result<String> {
    let yaml = serde_yaml::to_string(data)?;
    Ok(yaml)
}

pub fn update_markdown_frontmatter<T: Serialize>(frontmatter: &T, content: &str) -> Result<String> {
    let yaml = serialize_yaml_frontmatter(frontmatter)?;
    Ok(format!("---\n{}\n---\n{}", yaml, content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestFrontmatter {
        title: String,
        tags: Vec<String>,
    }

    #[test]
    fn test_extract_frontmatter() {
        let markdown = r#"---
title: Test
tags:
  - a
  - b
---

# Content"#;

        let (frontmatter, content) = extract_frontmatter::<TestFrontmatter>(markdown).unwrap();
        assert_eq!(frontmatter.title, "Test");
        assert_eq!(frontmatter.tags, vec!["a", "b"]);
        assert_eq!(content, "# Content");
    }

    #[test]
    fn test_update_markdown_frontmatter() {
        let frontmatter = TestFrontmatter {
            title: "Test".to_string(),
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let content = "# Content";

        let result = update_markdown_frontmatter(&frontmatter, content).unwrap();
        assert!(result.contains("title: Test"));
        assert!(result.contains("# Content"));
    }
}
