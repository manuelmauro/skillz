//! Command-line interface definitions.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Main CLI application.
#[derive(Parser)]
#[command(name = "skilo")]
#[command(author, version, about = "CLI tool for Agent Skills development", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The subcommand to run.
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

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Command {
    /// Install skills from a git repository or local path
    ///
    /// Supports various source formats:
    ///   owner/repo                     GitHub shorthand
    ///   `https://github.com/owner/repo`  Full URL
    ///   git@github.com:owner/repo.git  SSH URL
    ///   ./path/to/skills               Local path
    #[command(verbatim_doc_comment)]
    Add(AddArgs),

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

    /// Read skill properties as JSON
    ///
    /// Outputs skill metadata including name, description, license,
    /// compatibility, metadata, and allowed_tools for one or more skills.
    #[command(verbatim_doc_comment)]
    ReadProperties(ReadPropertiesArgs),

    /// Generate XML prompt for available skills
    ///
    /// Outputs an <available_skills> XML block suitable for use in
    /// agent system prompts, containing skill names, descriptions,
    /// and file locations.
    #[command(verbatim_doc_comment)]
    ToPrompt(ToPromptArgs),
}

/// Arguments for the `add` command.
#[derive(clap::Args, Clone)]
pub struct AddArgs {
    /// Source to install skills from (e.g., owner/repo, URL, or path)
    pub source: String,

    /// Install specific skill(s) by name
    #[arg(long, short)]
    pub skill: Option<Vec<String>>,

    /// List available skills without installing
    #[arg(long, short)]
    pub list: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,

    /// Specify git branch
    #[arg(long, short)]
    pub branch: Option<String>,

    /// Specify git tag
    #[arg(long, short = 't')]
    pub tag: Option<String>,

    /// Target agent (determines install directory)
    #[arg(long, short, value_enum)]
    pub agent: Option<Agent>,

    /// Custom output directory
    #[arg(long, short, overrides_with = "agent")]
    pub output: Option<std::path::PathBuf>,
}

/// Supported AI coding agents (CLI enum).
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Agent {
    /// OpenCode
    OpenCode,
    /// Claude Code
    Claude,
    /// Codex
    Codex,
    /// Cursor
    Cursor,
    /// Amp
    Amp,
    /// Kilo Code
    KiloCode,
    /// Roo Code
    RooCode,
    /// Goose
    Goose,
    /// Gemini CLI
    Gemini,
    /// Antigravity
    Antigravity,
    /// GitHub Copilot
    Copilot,
    /// Clawdbot
    Clawdbot,
    /// Droid
    Droid,
    /// Windsurf
    Windsurf,
}

impl From<Agent> for crate::agent::Agent {
    fn from(agent: Agent) -> Self {
        match agent {
            Agent::OpenCode => Self::OpenCode,
            Agent::Claude => Self::Claude,
            Agent::Codex => Self::Codex,
            Agent::Cursor => Self::Cursor,
            Agent::Amp => Self::Amp,
            Agent::KiloCode => Self::KiloCode,
            Agent::RooCode => Self::RooCode,
            Agent::Goose => Self::Goose,
            Agent::Gemini => Self::Gemini,
            Agent::Antigravity => Self::Antigravity,
            Agent::Copilot => Self::Copilot,
            Agent::Clawdbot => Self::Clawdbot,
            Agent::Droid => Self::Droid,
            Agent::Windsurf => Self::Windsurf,
        }
    }
}

/// Arguments for the `new` command.
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

/// Arguments for the `lint` command.
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

/// Arguments for the `fmt` command.
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

/// Arguments for the `check` command.
#[derive(clap::Args, Clone)]
pub struct CheckArgs {
    /// Path to skill or directory containing skills
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for the `read-properties` command.
#[derive(clap::Args, Clone)]
pub struct ReadPropertiesArgs {
    /// Paths to skills or directories containing skills
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,
}

/// Arguments for the `to-prompt` command.
#[derive(clap::Args, Clone)]
pub struct ToPromptArgs {
    /// Paths to skills or directories containing skills
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,
}

/// Output format for command results.
#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// JSON output.
    Json,
    /// SARIF output for code scanning integrations.
    Sarif,
}

/// Available skill templates.
#[derive(ValueEnum, Clone, Copy, Default, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Template {
    /// Minimal working skill with a greeting script.
    #[default]
    HelloWorld,
    /// Bare-bones skill with only SKILL.md.
    Minimal,
    /// Complete skill with all optional directories.
    Full,
    /// Skill focused on script execution.
    ScriptBased,
}

/// Supported script languages.
#[derive(ValueEnum, Clone, Copy, Default, Debug)]
pub enum ScriptLang {
    /// Python scripts.
    #[default]
    Python,
    /// Bash scripts.
    Bash,
    /// JavaScript scripts.
    Javascript,
    /// TypeScript scripts.
    Typescript,
}
