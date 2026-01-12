use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum SkillzError {
    #[error("Skill '{name}' already exists at {path}")]
    #[diagnostic(code(skilo::skill_exists))]
    SkillExists { name: String, path: String },

    #[error(
        "Invalid skill name '{0}': must be 1-64 lowercase alphanumeric chars with single hyphens"
    )]
    #[diagnostic(code(skilo::invalid_name))]
    InvalidName(String),

    #[error("No skills found in {path}")]
    #[diagnostic(code(skilo::no_skills))]
    NoSkillsFound { path: String },

    #[error("Configuration error: {0}")]
    #[diagnostic(code(skilo::config))]
    Config(String),

    #[error("Validation failed with {0} error(s)")]
    #[diagnostic(code(skilo::validation_failed))]
    ValidationFailed(usize),

    #[error("Format check failed: {0} file(s) need formatting")]
    #[diagnostic(code(skilo::format_failed))]
    FormatCheckFailed(usize),

    #[error("Manifest error: {0}")]
    #[diagnostic(code(skilo::manifest))]
    Manifest(#[from] crate::skill::manifest::ManifestError),

    #[error("IO error: {0}")]
    #[diagnostic(code(skilo::io))]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SkillzError>;
