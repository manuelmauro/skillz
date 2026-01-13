use crate::skill::manifest::Manifest;
use crate::skill::rules::Rule;
use crate::skill::validator::{Diagnostic, DiagnosticCode};

/// W002: Warns if scripts are not executable
pub struct ScriptExecutableRule;

impl Rule for ScriptExecutableRule {
    fn name(&self) -> &'static str {
        "script-executable"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let Some(skill_dir) = manifest.path.parent() else {
            return Vec::new();
        };

        let scripts_dir = skill_dir.join("scripts");
        if !scripts_dir.exists() {
            return Vec::new();
        }

        let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = path.metadata() {
                    if meta.permissions().mode() & 0o111 == 0 {
                        diagnostics.push(Diagnostic {
                            path: path.display().to_string(),
                            line: None,
                            column: None,
                            message: "Script is not executable".into(),
                            code: DiagnosticCode::W002,
                            fix_hint: Some(format!("Run: chmod +x {}", path.display())),
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

/// W003: Warns if scripts are missing shebang
pub struct ScriptShebangRule;

impl Rule for ScriptShebangRule {
    fn name(&self) -> &'static str {
        "script-shebang"
    }

    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic> {
        let Some(skill_dir) = manifest.path.parent() else {
            return Vec::new();
        };

        let scripts_dir = skill_dir.join("scripts");
        if !scripts_dir.exists() {
            return Vec::new();
        }

        let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path) {
                if !content.starts_with("#!") {
                    diagnostics.push(Diagnostic {
                        path: path.display().to_string(),
                        line: Some(1),
                        column: Some(1),
                        message: "Script missing shebang line".into(),
                        code: DiagnosticCode::W003,
                        fix_hint: Some("Add #!/usr/bin/env <interpreter> as first line".into()),
                    });
                }
            }
        }

        diagnostics
    }
}
