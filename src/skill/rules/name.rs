use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};
use once_cell::sync::Lazy;
use regex::Regex;

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

/// E001: Validates name format (lowercase alphanumeric + single hyphens)
pub struct NameFormatRule;

impl Rule for NameFormatRule {
    fn name(&self) -> &'static str {
        "name-format"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let name = &manifest.frontmatter.name;

        if NAME_REGEX.is_match(name) {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(2),
            column: Some(7),
            message: format!(
                "Invalid name '{}': must be lowercase alphanumeric with single hyphens",
                name
            ),
            code: DiagnosticCode::E001,
            fix_hint: Some("Use only lowercase letters, numbers, and single hyphens".into()),
        }]
    }
}

/// E002: Validates name length
pub struct NameLengthRule {
    max_length: usize,
}

impl NameLengthRule {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Rule for NameLengthRule {
    fn name(&self) -> &'static str {
        "name-length"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let name = &manifest.frontmatter.name;

        if name.len() <= self.max_length {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(2),
            column: Some(7),
            message: format!(
                "Name too long ({} chars, max {})",
                name.len(),
                self.max_length
            ),
            code: DiagnosticCode::E002,
            fix_hint: None,
        }]
    }
}

/// E003: Validates name matches parent directory
pub struct NameDirectoryRule;

impl Rule for NameDirectoryRule {
    fn name(&self) -> &'static str {
        "name-directory"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let name = &manifest.frontmatter.name;

        let Some(parent) = manifest.path.parent() else {
            return Vec::new();
        };

        let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) else {
            return Vec::new();
        };

        if dir_name == name {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(2),
            column: Some(7),
            message: format!(
                "Name '{}' does not match directory name '{}'",
                name, dir_name
            ),
            code: DiagnosticCode::E003,
            fix_hint: Some(format!(
                "Rename to '{}' or move to '{}/SKILL.md'",
                dir_name, name
            )),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(NAME_REGEX.is_match("my-skill"));
        assert!(NAME_REGEX.is_match("skill123"));
        assert!(NAME_REGEX.is_match("a"));
        assert!(NAME_REGEX.is_match("my-cool-skill"));
    }

    #[test]
    fn test_invalid_names() {
        assert!(!NAME_REGEX.is_match("My-Skill")); // uppercase
        assert!(!NAME_REGEX.is_match("-skill")); // leading hyphen
        assert!(!NAME_REGEX.is_match("skill-")); // trailing hyphen
        assert!(!NAME_REGEX.is_match("my--skill")); // consecutive hyphens
        assert!(!NAME_REGEX.is_match("my_skill")); // underscore
        assert!(!NAME_REGEX.is_match("")); // empty
    }
}
