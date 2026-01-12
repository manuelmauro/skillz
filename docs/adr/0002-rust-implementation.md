# ADR 0002: Rust Implementation

## Status

Proposed

## Context

[ADR 0001](./0001-skilo-cli-tool.md) defines the architecture and commands for the skilo CLI tool. This ADR describes the Rust implementation details, including crate dependencies, module organization, key data structures, and design patterns.

## Decision

### Crate Dependencies

```toml
[package]
name = "skilo"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "CLI tool for Agent Skills development"
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/skilo"
keywords = ["agent", "skills", "cli", "ai"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# CLI
clap = { version = "4", features = ["derive", "env", "wrap_help"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
toml = "0.8"

# Markdown
pulldown-cmark = "0.10"

# File system
walkdir = "2"
glob = "0.3"

# Output
colored = "2"
serde_json = "1"

# Error handling
thiserror = "1"
miette = { version = "7", features = ["fancy"] }

# Utilities
regex = "1"
once_cell = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
insta = { version = "1", features = ["yaml"] }
```

### Module Structure

```
src/
├── main.rs
├── lib.rs
├── cli.rs
├── commands/
│   ├── mod.rs
│   ├── new.rs
│   ├── lint.rs
│   ├── fmt.rs
│   └── check.rs
├── skill/
│   ├── mod.rs
│   ├── manifest.rs
│   ├── frontmatter.rs
│   ├── validator.rs
│   └── discovery.rs
├── templates/
│   ├── mod.rs
│   ├── hello_world.rs
│   ├── minimal.rs
│   ├── full.rs
│   └── script_based.rs
├── lang.rs
├── config.rs
├── output/
│   ├── mod.rs
│   ├── text.rs
│   ├── json.rs
│   └── sarif.rs
└── error.rs
```

### Core Types

#### CLI Definition (`src/cli.rs`)

```rust
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skilo")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Configuration file path
    #[arg(long, global = true, env = "SKILLZ_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Suppress non-error output
    #[arg(long, short, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new skill from a template
    New(NewArgs),

    /// Validate skills against the specification
    Lint(LintArgs),

    /// Format SKILL.md files
    Fmt(FmtArgs),

    /// Run all validations (lint + format check)
    Check(CheckArgs),

    /// Alias for lint --strict
    Validate(LintArgs),
}

#[derive(clap::Args)]
pub struct NewArgs {
    /// Name of the skill to create
    pub name: String,

    /// Template to use
    #[arg(long, short, default_value = "hello-world")]
    pub template: Template,

    /// Preferred script language
    #[arg(long, default_value = "python")]
    pub lang: ScriptLang,

    /// License for the skill
    #[arg(long)]
    pub license: Option<String>,

    /// Skill description
    #[arg(long, short)]
    pub description: Option<String>,

    /// Skip creating optional directories
    #[arg(long)]
    pub no_optional_dirs: bool,

    /// Skip creating scripts directory
    #[arg(long)]
    pub no_scripts: bool,

    /// Output directory (defaults to current directory)
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct LintArgs {
    /// Path to skill or directory containing skills
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,

    /// Auto-fix simple issues
    #[arg(long)]
    pub fix: bool,
}

#[derive(clap::Args)]
pub struct FmtArgs {
    /// Path to skill or directory containing skills
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Check formatting without modifying
    #[arg(long)]
    pub check: bool,

    /// Show diff of changes
    #[arg(long)]
    pub diff: bool,
}

#[derive(clap::Args)]
pub struct CheckArgs {
    /// Path to skill or directory containing skills
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(ValueEnum, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(ValueEnum, Clone, Copy, Default)]
pub enum Template {
    #[default]
    HelloWorld,
    Minimal,
    Full,
    ScriptBased,
}

#[derive(ValueEnum, Clone, Copy, Default)]
pub enum ScriptLang {
    #[default]
    Python,
    Bash,
    Javascript,
    Typescript,
}
```

#### Script Language (`src/lang.rs`)

```rust
use crate::cli::ScriptLang;

impl ScriptLang {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Python => "py",
            Self::Bash => "sh",
            Self::Javascript => "js",
            Self::Typescript => "ts",
        }
    }

    pub fn shebang(&self) -> &'static str {
        match self {
            Self::Python => "#!/usr/bin/env python3",
            Self::Bash => "#!/usr/bin/env bash",
            Self::Javascript => "#!/usr/bin/env node",
            Self::Typescript => "#!/usr/bin/env -S npx ts-node",
        }
    }

    pub fn comment_prefix(&self) -> &'static str {
        match self {
            Self::Python => "#",
            Self::Bash => "#",
            Self::Javascript => "//",
            Self::Typescript => "//",
        }
    }

    pub fn file_name(&self, name: &str) -> String {
        format!("{}.{}", name, self.extension())
    }
}
```

#### Skill Frontmatter (`src/skill/frontmatter.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frontmatter {
    /// Skill name (required, 1-64 chars, lowercase alphanumeric + hyphens)
    pub name: String,

    /// Skill description (required, 1-1024 chars)
    pub description: String,

    /// License identifier or file reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Compatibility requirements (max 500 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Additional metadata key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Pre-approved tools (space-delimited)
    #[serde(rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,
}

impl Frontmatter {
    /// Canonical key ordering for formatting
    pub const KEY_ORDER: &'static [&'static str] = &[
        "name",
        "description",
        "license",
        "compatibility",
        "metadata",
        "allowed-tools",
    ];
}
```

#### Skill Manifest (`src/skill/manifest.rs`)

```rust
use crate::skill::frontmatter::Frontmatter;
use std::path::PathBuf;

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

impl Manifest {
    /// Parse a SKILL.md file
    pub fn parse(path: PathBuf) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(&path)?;
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
        let body = content[body_start..].trim_start().to_string();

        // Count lines to frontmatter end
        let body_start_line = content[..body_start].lines().count() + 1;

        Ok((frontmatter, body, body_start_line))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("SKILL.md must start with YAML frontmatter (---)")]
    MissingFrontmatter,

    #[error("Frontmatter is not closed (missing closing ---)")]
    UnclosedFrontmatter,

    #[error("Invalid YAML in frontmatter: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Validation (`src/skill/validator.rs`)

```rust
use crate::skill::manifest::Manifest;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

static NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_ok_strict(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: DiagnosticCode,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticCode {
    // Errors
    E001, // Invalid name format
    E002, // Name too long
    E003, // Name mismatch with directory
    E004, // Missing description
    E005, // Description too long
    E006, // Compatibility too long
    E007, // Invalid YAML
    E008, // Missing SKILL.md
    E009, // Referenced file not found

    // Warnings
    W001, // Body exceeds 500 lines
    W002, // Script not executable
    W003, // Script missing shebang
    W004, // Empty optional directory
}

pub struct Validator;

impl Validator {
    pub fn validate(manifest: &Manifest) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate name
        Self::validate_name(manifest, &mut errors);

        // Validate description
        Self::validate_description(manifest, &mut errors);

        // Validate compatibility
        Self::validate_compatibility(manifest, &mut errors);

        // Validate body length
        Self::validate_body(manifest, &mut warnings);

        // Validate file references
        Self::validate_references(manifest, &mut errors);

        // Validate scripts
        Self::validate_scripts(manifest, &mut warnings);

        ValidationResult { errors, warnings }
    }

    fn validate_name(manifest: &Manifest, errors: &mut Vec<Diagnostic>) {
        let name = &manifest.frontmatter.name;
        let path_str = manifest.path.display().to_string();

        // Check format
        if !NAME_REGEX.is_match(name) {
            errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(2),
                column: Some(7),
                message: format!(
                    "Invalid name '{}': must be lowercase alphanumeric with single hyphens",
                    name
                ),
                code: DiagnosticCode::E001,
                fix_hint: Some("Use only lowercase letters, numbers, and single hyphens".into()),
            });
        }

        // Check length
        if name.len() > 64 {
            errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(2),
                column: Some(7),
                message: format!("Name too long ({} chars, max 64)", name.len()),
                code: DiagnosticCode::E002,
                fix_hint: None,
            });
        }

        // Check directory match
        if let Some(parent) = manifest.path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                if dir_name != name {
                    errors.push(Diagnostic {
                        path: path_str,
                        line: Some(2),
                        column: Some(7),
                        message: format!(
                            "Name '{}' does not match directory name '{}'",
                            name, dir_name
                        ),
                        code: DiagnosticCode::E003,
                        fix_hint: Some(format!("Rename to '{}' or move to '{}/SKILL.md'", dir_name, name)),
                    });
                }
            }
        }
    }

    fn validate_description(manifest: &Manifest, errors: &mut Vec<Diagnostic>) {
        let desc = &manifest.frontmatter.description;
        let path_str = manifest.path.display().to_string();

        if desc.is_empty() {
            errors.push(Diagnostic {
                path: path_str.clone(),
                line: Some(3),
                column: Some(14),
                message: "Description cannot be empty".into(),
                code: DiagnosticCode::E004,
                fix_hint: None,
            });
        }

        if desc.len() > 1024 {
            errors.push(Diagnostic {
                path: path_str,
                line: Some(3),
                column: Some(14),
                message: format!("Description too long ({} chars, max 1024)", desc.len()),
                code: DiagnosticCode::E005,
                fix_hint: None,
            });
        }
    }

    fn validate_compatibility(manifest: &Manifest, errors: &mut Vec<Diagnostic>) {
        if let Some(compat) = &manifest.frontmatter.compatibility {
            if compat.len() > 500 {
                errors.push(Diagnostic {
                    path: manifest.path.display().to_string(),
                    line: None,
                    column: None,
                    message: format!("Compatibility too long ({} chars, max 500)", compat.len()),
                    code: DiagnosticCode::E006,
                    fix_hint: None,
                });
            }
        }
    }

    fn validate_body(manifest: &Manifest, warnings: &mut Vec<Diagnostic>) {
        let line_count = manifest.body.lines().count();
        if line_count > 500 {
            warnings.push(Diagnostic {
                path: manifest.path.display().to_string(),
                line: Some(manifest.body_start_line + 500),
                column: None,
                message: format!(
                    "Body exceeds recommended 500 lines ({} lines). Consider using references/",
                    line_count
                ),
                code: DiagnosticCode::W001,
                fix_hint: Some("Move detailed content to references/ directory".into()),
            });
        }
    }

    fn validate_references(manifest: &Manifest, errors: &mut Vec<Diagnostic>) {
        // Extract file references from body using regex
        static REF_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"`((?:scripts|references|assets)/[^`]+)`").unwrap());

        let skill_dir = manifest.path.parent().unwrap();

        for cap in REF_REGEX.captures_iter(&manifest.body) {
            let ref_path = &cap[1];
            let full_path = skill_dir.join(ref_path);

            if !full_path.exists() {
                errors.push(Diagnostic {
                    path: manifest.path.display().to_string(),
                    line: None,
                    column: None,
                    message: format!("Referenced file not found: {}", ref_path),
                    code: DiagnosticCode::E009,
                    fix_hint: Some(format!("Create {} or remove the reference", ref_path)),
                });
            }
        }
    }

    fn validate_scripts(manifest: &Manifest, warnings: &mut Vec<Diagnostic>) {
        let skill_dir = manifest.path.parent().unwrap();
        let scripts_dir = skill_dir.join("scripts");

        if !scripts_dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                // Check executable permission (Unix only)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = path.metadata() {
                        if meta.permissions().mode() & 0o111 == 0 {
                            warnings.push(Diagnostic {
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

                // Check shebang
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if !content.starts_with("#!") {
                        warnings.push(Diagnostic {
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
        }
    }
}
```

#### Skill Discovery (`src/skill/discovery.rs`)

```rust
use crate::skill::manifest::{Manifest, ManifestError};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Discovery;

impl Discovery {
    /// Find all skills in a directory tree
    pub fn find_skills(root: &Path) -> Vec<PathBuf> {
        if root.is_file() && root.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
            return vec![root.to_path_buf()];
        }

        let skill_md = root.join("SKILL.md");
        if skill_md.exists() {
            return vec![skill_md];
        }

        WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "SKILL.md")
            .map(|e| e.into_path())
            .collect()
    }

    /// Load all skills from paths
    pub fn load_skills(paths: &[PathBuf]) -> Vec<Result<Manifest, (PathBuf, ManifestError)>> {
        paths
            .iter()
            .map(|path| {
                Manifest::parse(path.clone()).map_err(|e| (path.clone(), e))
            })
            .collect()
    }
}
```

#### Error Types (`src/error.rs`)

```rust
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum SkillzError {
    #[error("Skill '{name}' already exists at {path}")]
    #[diagnostic(code(skilo::skill_exists))]
    SkillExists { name: String, path: String },

    #[error("Invalid skill name: {0}")]
    #[diagnostic(code(skilo::invalid_name))]
    InvalidName(String),

    #[error("No skills found in {path}")]
    #[diagnostic(code(skilo::no_skills))]
    NoSkillsFound { path: String },

    #[error("Configuration error: {0}")]
    #[diagnostic(code(skilo::config))]
    Config(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Manifest(#[from] crate::skill::manifest::ManifestError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SkillzError>;
```

#### Configuration (`src/config.rs`)

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lint: LintConfig,
    pub fmt: FmtConfig,
    pub new: NewConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LintConfig {
    pub strict: bool,
    pub max_body_lines: usize,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            strict: false,
            max_body_lines: 500,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FmtConfig {
    pub sort_frontmatter: bool,
    pub indent_size: usize,
}

impl Default for FmtConfig {
    fn default() -> Self {
        Self {
            sort_frontmatter: true,
            indent_size: 2,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct NewConfig {
    pub default_license: Option<String>,
    pub default_template: String,
    pub default_lang: String,
}

impl Default for NewConfig {
    fn default() -> Self {
        Self {
            default_license: None,
            default_template: "hello-world".into(),
            default_lang: "python".into(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Result<Self, std::io::Error> {
        let config_path = path
            .cloned()
            .or_else(|| Self::find_config())
            .unwrap_or_else(|| PathBuf::from(".skilorc.toml"));

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }

    fn find_config() -> Option<PathBuf> {
        let candidates = [".skilorc.toml", "skilo.toml", ".skilo/config.toml"];

        for name in candidates {
            let path = PathBuf::from(name);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}
```

#### Template Rendering (`src/templates/mod.rs`)

```rust
use crate::cli::{ScriptLang, Template};
use std::path::Path;

mod hello_world;
mod minimal;
mod full;
mod script_based;

pub struct TemplateContext {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub lang: ScriptLang,
    pub include_optional_dirs: bool,
    pub include_scripts: bool,
}

pub trait SkillTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()>;
}

pub fn get_template(template: Template) -> Box<dyn SkillTemplate> {
    match template {
        Template::HelloWorld => Box::new(hello_world::HelloWorldTemplate),
        Template::Minimal => Box::new(minimal::MinimalTemplate),
        Template::Full => Box::new(full::FullTemplate),
        Template::ScriptBased => Box::new(script_based::ScriptBasedTemplate),
    }
}
```

#### Hello World Template (`src/templates/hello_world.rs`)

```rust
use super::{SkillTemplate, TemplateContext};
use crate::cli::ScriptLang;
use std::fs;
use std::path::Path;

pub struct HelloWorldTemplate;

impl SkillTemplate for HelloWorldTemplate {
    fn render(&self, ctx: &TemplateContext, output_dir: &Path) -> std::io::Result<()> {
        let skill_dir = output_dir.join(&ctx.name);
        fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md
        let skill_md = self.render_skill_md(ctx);
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;

        // Write script
        if ctx.include_scripts {
            let scripts_dir = skill_dir.join("scripts");
            fs::create_dir_all(&scripts_dir)?;

            let script_name = ctx.lang.file_name("greet");
            let script_content = self.render_script(ctx);
            let script_path = scripts_dir.join(&script_name);

            fs::write(&script_path, script_content)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms)?;
            }
        }

        Ok(())
    }
}

impl HelloWorldTemplate {
    fn render_skill_md(&self, ctx: &TemplateContext) -> String {
        let mut frontmatter = format!(
            "---\nname: {}\ndescription: {}\n",
            ctx.name, ctx.description
        );

        if let Some(license) = &ctx.license {
            frontmatter.push_str(&format!("license: {}\n", license));
        }

        frontmatter.push_str("---\n\n");

        let title = ctx
            .name
            .split('-')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().chain(c).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let body = format!(
            r#"# {}

This skill provides a simple greeting functionality.

## Usage

Run the greeting script to display a personalized message.

## Scripts

- `scripts/greet.{}` - Outputs a greeting message

## Example

```bash
./scripts/greet.{} World
# Output: Hello, World!
```
"#,
            title,
            ctx.lang.extension(),
            ctx.lang.extension()
        );

        frontmatter + &body
    }

    fn render_script(&self, ctx: &TemplateContext) -> String {
        match ctx.lang {
            ScriptLang::Python => format!(
                r#"{}
"""A simple greeting script."""

import sys


def main():
    name = sys.argv[1] if len(sys.argv) > 1 else "World"
    print(f"Hello, {{name}}!")


if __name__ == "__main__":
    main()
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Bash => format!(
                r#"{}
# A simple greeting script.

set -euo pipefail

name="${{1:-World}}"
echo "Hello, ${{name}}!"
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Javascript => format!(
                r#"{}
// A simple greeting script.

const name = process.argv[2] || "World";
console.log(`Hello, ${{name}}!`);
"#,
                ctx.lang.shebang()
            ),

            ScriptLang::Typescript => format!(
                r#"{}
// A simple greeting script.

const name: string = process.argv[2] || "World";
console.log(`Hello, ${{name}}!`);
"#,
                ctx.lang.shebang()
            ),
        }
    }
}
```

#### Main Entry Point (`src/main.rs`)

```rust
use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod error;
mod lang;
mod output;
mod skill;
mod templates;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load(cli.config.as_ref())
        .map_err(|e| miette::miette!("Failed to load config: {}", e))?;

    let exit_code = match cli.command {
        Command::New(args) => commands::new::run(args, &config, &cli)?,
        Command::Lint(args) => commands::lint::run(args, &config, &cli)?,
        Command::Fmt(args) => commands::fmt::run(args, &config, &cli)?,
        Command::Check(args) => commands::check::run(args, &config, &cli)?,
        Command::Validate(args) => {
            let mut args = args;
            args.strict = true;
            commands::lint::run(args, &config, &cli)?
        }
    };

    std::process::exit(exit_code);
}
```

### Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        assert!(NAME_REGEX.is_match("my-skill"));
        assert!(NAME_REGEX.is_match("skill123"));
        assert!(NAME_REGEX.is_match("a"));
    }

    #[test]
    fn test_invalid_name() {
        assert!(!NAME_REGEX.is_match("My-Skill")); // uppercase
        assert!(!NAME_REGEX.is_match("-skill"));   // leading hyphen
        assert!(!NAME_REGEX.is_match("skill-"));   // trailing hyphen
        assert!(!NAME_REGEX.is_match("my--skill")); // consecutive hyphens
        assert!(!NAME_REGEX.is_match("my_skill")); // underscore
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill
"#;
        let manifest = Manifest::parse_content(
            PathBuf::from("test-skill/SKILL.md"),
            content
        ).unwrap();

        assert_eq!(manifest.frontmatter.name, "test-skill");
        assert_eq!(manifest.frontmatter.description, "A test skill");
    }
}
```

#### Integration Tests

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_new_creates_skill() {
    let temp = TempDir::new().unwrap();

    Command::cargo_bin("skilo")
        .unwrap()
        .args(["new", "my-skill", "--lang", "bash"])
        .current_dir(&temp)
        .assert()
        .success();

    assert!(temp.path().join("my-skill/SKILL.md").exists());
    assert!(temp.path().join("my-skill/scripts/greet.sh").exists());
}

#[test]
fn test_lint_valid_skill() {
    let temp = TempDir::new().unwrap();

    // Create valid skill
    Command::cargo_bin("skilo")
        .unwrap()
        .args(["new", "valid-skill"])
        .current_dir(&temp)
        .assert()
        .success();

    // Lint should pass
    Command::cargo_bin("skilo")
        .unwrap()
        .args(["lint", "valid-skill"])
        .current_dir(&temp)
        .assert()
        .success();
}

#[test]
fn test_lint_invalid_name() {
    let temp = TempDir::new().unwrap();
    let skill_dir = temp.path().join("Invalid_Skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Invalid_Skill\ndescription: Bad\n---\n",
    ).unwrap();

    Command::cargo_bin("skilo")
        .unwrap()
        .args(["lint", "Invalid_Skill"])
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("E001"));
}
```

#### Snapshot Tests

```rust
// tests/snapshots.rs
use insta::assert_yaml_snapshot;

#[test]
fn test_hello_world_template_python() {
    let ctx = TemplateContext {
        name: "my-skill".into(),
        description: "A test skill".into(),
        license: Some("MIT".into()),
        lang: ScriptLang::Python,
        include_optional_dirs: false,
        include_scripts: true,
    };

    let template = HelloWorldTemplate;
    let skill_md = template.render_skill_md(&ctx);

    assert_yaml_snapshot!(skill_md);
}
```

## Consequences

### Positive

- Type-safe CLI with compile-time validation via clap derive
- Rich error messages with miette diagnostics
- Comprehensive test coverage with multiple strategies
- Clear separation of concerns between modules
- Extensible template system

### Negative

- Larger binary size due to dependencies (mitigated by LTO)
- Compile times increased by derive macros

### Neutral

- Follows Rust ecosystem conventions
- Compatible with cargo install distribution

## References

- [Clap Derive Documentation](https://docs.rs/clap/latest/clap/_derive/)
- [Miette Error Handling](https://docs.rs/miette/latest/miette/)
- [Insta Snapshot Testing](https://insta.rs/)
