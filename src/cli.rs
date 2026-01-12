use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skilo")]
#[command(author, version, about = "CLI tool for Agent Skills development", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Configuration file path
    #[arg(long, global = true, env = "SKILO_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, default_value = "text", value_enum)]
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
    ///
    /// Skills must be directories containing a SKILL.md file with valid frontmatter.
    /// Example structure:
    ///   my-skill/
    ///     SKILL.md      # Required: contains name, description in YAML frontmatter
    ///     scripts/      # Optional: executable scripts
    ///     tests/        # Optional: test files
    #[command(verbatim_doc_comment)]
    Lint(LintArgs),

    /// Format SKILL.md files
    Fmt(FmtArgs),

    /// Run all validations (lint + format check)
    Check(CheckArgs),

    /// Alias for lint --strict
    Validate(LintArgs),
}

#[derive(clap::Args, Clone)]
pub struct NewArgs {
    /// Name of the skill to create
    pub name: String,

    /// Template to use
    #[arg(long, short, default_value = "hello-world", value_enum)]
    pub template: Template,

    /// Preferred script language
    #[arg(long, default_value = "python", value_enum)]
    pub lang: ScriptLang,

    /// License for the skill (SPDX identifier)
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

#[derive(clap::Args, Clone)]
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

#[derive(clap::Args, Clone)]
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

#[derive(clap::Args, Clone)]
pub struct CheckArgs {
    /// Path to skill or directory containing skills
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Template {
    #[default]
    HelloWorld,
    Minimal,
    Full,
    ScriptBased,
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum ScriptLang {
    #[default]
    Python,
    Bash,
    Javascript,
    Typescript,
}
