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

    /// List installed skills
    ///
    /// Shows skills installed at project or global level.
    #[command(verbatim_doc_comment)]
    List(ListArgs),

    /// List detected agents
    ///
    /// Shows AI coding agents detected in the current project or globally,
    /// along with their skill counts and feature support.
    #[command(verbatim_doc_comment)]
    Agents(AgentsArgs),

    /// Manage the git cache
    ///
    /// Skilo caches git repositories in ~/.skilo/git/ to speed up
    /// repeated installs and enable offline usage.
    #[command(verbatim_doc_comment)]
    Cache(CacheArgs),
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

    /// Target agent(s) (determines install directory)
    ///
    /// Can be specified multiple times: --agent claude --agent cursor
    /// Use 'all' to install to all detected agents.
    #[arg(long, short, value_enum)]
    pub agent: Option<Vec<Agent>>,

    /// Install to global skills directory (~/.claude/skills/)
    #[arg(long, short = 'g')]
    pub global: bool,

    /// Custom output directory
    #[arg(long, short, conflicts_with_all = ["agent", "global"])]
    pub output: Option<std::path::PathBuf>,
}

/// Represents a CLI agent selection: either all agents or a specific one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSelection {
    /// All detected agents.
    All,
    /// A single specific agent.
    Single(crate::agent::Agent),
}

/// Supported AI coding agents (CLI enum).
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    /// All detected agents
    All,
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

impl Agent {
    /// Convert to an agent selection.
    pub fn to_selection(&self) -> AgentSelection {
        match self {
            Agent::All => AgentSelection::All,
            Agent::OpenCode => AgentSelection::Single(crate::agent::Agent::OpenCode),
            Agent::Claude => AgentSelection::Single(crate::agent::Agent::Claude),
            Agent::Codex => AgentSelection::Single(crate::agent::Agent::Codex),
            Agent::Cursor => AgentSelection::Single(crate::agent::Agent::Cursor),
            Agent::Amp => AgentSelection::Single(crate::agent::Agent::Amp),
            Agent::KiloCode => AgentSelection::Single(crate::agent::Agent::KiloCode),
            Agent::RooCode => AgentSelection::Single(crate::agent::Agent::RooCode),
            Agent::Goose => AgentSelection::Single(crate::agent::Agent::Goose),
            Agent::Gemini => AgentSelection::Single(crate::agent::Agent::Gemini),
            Agent::Antigravity => AgentSelection::Single(crate::agent::Agent::Antigravity),
            Agent::Copilot => AgentSelection::Single(crate::agent::Agent::Copilot),
            Agent::Clawdbot => AgentSelection::Single(crate::agent::Agent::Clawdbot),
            Agent::Droid => AgentSelection::Single(crate::agent::Agent::Droid),
            Agent::Windsurf => AgentSelection::Single(crate::agent::Agent::Windsurf),
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

    /// Target agent (determines output directory)
    #[arg(long, short, value_enum)]
    pub agent: Option<Agent>,

    /// Create skill in global skills directory
    #[arg(long, short = 'g')]
    pub global: bool,

    /// Output directory (defaults to agent skills directory)
    #[arg(long, short, conflicts_with_all = ["agent", "global"])]
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

/// Arguments for the `list` command.
#[derive(clap::Args, Clone)]
pub struct ListArgs {
    /// Project directory to list skills from
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// List global skills only
    #[arg(long, short = 'g')]
    pub global: bool,

    /// List all skills (project + global)
    #[arg(long, conflicts_with = "global")]
    pub all: bool,

    /// Target agent
    #[arg(long, short, value_enum)]
    pub agent: Option<Agent>,
}

/// Arguments for the `agents` command.
#[derive(clap::Args, Clone)]
pub struct AgentsArgs {
    /// Show verbose output (feature support matrix)
    #[arg(long, short)]
    pub verbose: bool,
}

/// Arguments for the `cache` command.
#[derive(clap::Args, Clone)]
pub struct CacheArgs {
    /// Cache subcommand
    #[command(subcommand)]
    pub command: Option<CacheCommand>,
}

/// Cache subcommands.
#[derive(Subcommand, Clone)]
pub enum CacheCommand {
    /// Show cache location
    Path,

    /// Clean old checkouts
    Clean {
        /// Remove all cached data (db + checkouts)
        #[arg(long)]
        all: bool,

        /// Maximum age in days for checkouts (default: 30)
        #[arg(long, default_value = "30")]
        max_age: u32,
    },
}
