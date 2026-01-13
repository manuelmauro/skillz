use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};

/// W001: Warns if body exceeds max_body_lines
pub struct BodyLengthRule {
    max_lines: usize,
}

impl BodyLengthRule {
    pub fn new(max_lines: usize) -> Self {
        Self { max_lines }
    }
}

impl Rule for BodyLengthRule {
    fn name(&self) -> &'static str {
        "body-length"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let line_count = manifest.body.lines().count();

        if line_count <= self.max_lines {
            return Vec::new();
        }

        vec![Diagnostic {
            path: manifest.path.display().to_string(),
            line: Some(manifest.body_start_line + self.max_lines),
            column: None,
            message: format!(
                "Body exceeds recommended {} lines ({} lines). Consider using references/",
                self.max_lines, line_count
            ),
            code: DiagnosticCode::W001,
            fix_hint: Some("Move detailed content to references/ directory".into()),
        }]
    }
}
