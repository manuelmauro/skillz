use crate::skill::frontmatter::Frontmatter;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug)]
pub struct Manifest {
    /// Path to the SKILL.md file
    pub path: PathBuf,

    /// Parsed frontmatter
    pub frontmatter: Frontmatter,

    /// Raw frontmatter YAML string
    pub frontmatter_raw: String,

    /// Markdown body content
    pub body: String,

    /// Line number where body starts
    pub body_start_line: usize,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("SKILL.md must start with YAML frontmatter (---)")]
    MissingFrontmatter,

    #[error("Frontmatter is not closed (missing closing ---)")]
    UnclosedFrontmatter,

    #[error("Invalid YAML in frontmatter: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Manifest {
    /// Parse a SKILL.md file
    pub fn parse(path: PathBuf) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(&path).map_err(|e| ManifestError::Io {
            path: path.clone(),
            source: e,
        })?;
        Self::parse_content(path, &content)
    }

    /// Parse from string content
    pub fn parse_content(path: PathBuf, content: &str) -> Result<Self, ManifestError> {
        let (frontmatter_raw, body, body_start_line) = Self::split_content(content)?;
        let frontmatter: Frontmatter = serde_yaml::from_str(&frontmatter_raw)?;

        Ok(Self {
            path,
            frontmatter,
            frontmatter_raw,
            body,
            body_start_line,
        })
    }

    fn split_content(content: &str) -> Result<(String, String, usize), ManifestError> {
        let content = content.trim_start();

        if !content.starts_with("---") {
            return Err(ManifestError::MissingFrontmatter);
        }

        let after_open = &content[3..];
        let close_pos = after_open
            .find("\n---")
            .ok_or(ManifestError::UnclosedFrontmatter)?;

        let frontmatter = after_open[..close_pos].trim().to_string();
        let body_start = 3 + close_pos + 4; // "---" + content + "\n---"
        let body = if body_start < content.len() {
            content[body_start..].trim_start().to_string()
        } else {
            String::new()
        };

        // Count lines to frontmatter end
        let body_start_line = content[..body_start.min(content.len())].lines().count() + 1;

        Ok((frontmatter, body, body_start_line))
    }

    /// Reconstruct with reformatted frontmatter
    pub fn to_string_formatted(&self) -> Result<String, serde_yaml::Error> {
        let yaml = self.frontmatter.to_yaml()?;
        Ok(format!("---\n{}---\n\n{}", yaml, self.body))
    }
}

impl fmt::Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "---\n{}\n---\n\n{}",
            self.frontmatter_raw.trim(),
            self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_manifest() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

Some content here.
"#;
        let manifest =
            Manifest::parse_content(PathBuf::from("test-skill/SKILL.md"), content).unwrap();

        assert_eq!(manifest.frontmatter.name, "test-skill");
        assert_eq!(manifest.frontmatter.description, "A test skill");
        assert!(manifest.body.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# No frontmatter here";
        let result = Manifest::parse_content(PathBuf::from("test/SKILL.md"), content);
        assert!(matches!(result, Err(ManifestError::MissingFrontmatter)));
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let content = "---\nname: test\n# No closing";
        let result = Manifest::parse_content(PathBuf::from("test/SKILL.md"), content);
        assert!(matches!(result, Err(ManifestError::UnclosedFrontmatter)));
    }
}
