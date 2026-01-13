use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};

/// E004: Validates description is not empty
pub struct DescriptionRequiredRule;

impl Rule for DescriptionRequiredRule {
    fn name(&self) -> &'static str {
        "description-required"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let desc = &manifest.frontmatter.description;

        if !desc.is_empty() {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(3),
            column: Some(14),
            message: "Description cannot be empty".into(),
            code: DiagnosticCode::E004,
            fix_hint: None,
        }]
    }
}

/// E005: Validates description length
pub struct DescriptionLengthRule {
    max_length: usize,
}

impl DescriptionLengthRule {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Rule for DescriptionLengthRule {
    fn name(&self) -> &'static str {
        "description-length"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let desc = &manifest.frontmatter.description;

        if desc.len() <= self.max_length {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(3),
            column: Some(14),
            message: format!(
                "Description too long ({} chars, max {})",
                desc.len(),
                self.max_length
            ),
            code: DiagnosticCode::E005,
            fix_hint: None,
        }]
    }
}
