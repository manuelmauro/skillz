use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};
use once_cell::sync::Lazy;
use regex::Regex;

static REF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"`((?:scripts|references|assets)/[^`]+)`").unwrap());

/// E009: Validates that referenced files exist
pub struct ReferencesExistRule;

impl Rule for ReferencesExistRule {
    fn name(&self) -> &'static str {
        "references-exist"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let Some(skill_dir) = manifest.path.parent() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        for cap in REF_REGEX.captures_iter(&manifest.body) {
            let ref_path = &cap[1];
            let full_path = skill_dir.join(ref_path);

            if !full_path.exists() {
                diagnostics.push(Diagnostic {
                    path: manifest.path.display().to_string(),
                    line: None,
                    column: None,
                    message: format!("Referenced file not found: {}", ref_path),
                    code: DiagnosticCode::E009,
                    fix_hint: Some(format!("Create {} or remove the reference", ref_path)),
                });
            }
        }

        diagnostics
    }
}
