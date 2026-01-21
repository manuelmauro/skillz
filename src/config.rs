//! Configuration file handling.

use crate::agent::Agent;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

/// A configurable threshold that can be default, disabled, or a specific value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Threshold {
    /// Use the default value for this rule.
    #[default]
    Default,
    /// Rule is disabled.
    Disabled,
    /// Rule is enabled with a specific value.
    Value(usize),
}

impl Threshold {
    /// Resolve the threshold to an `Option<usize>` given a default value.
    pub fn resolve(self, default: usize) -> Option<usize> {
        match self {
            Self::Default => Some(default),
            Self::Disabled => None,
            Self::Value(n) => Some(n),
        }
    }
}

fn deserialize_threshold<'de, D>(deserializer: D) -> Result<Threshold, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Bool(bool),
        Number(usize),
    }

    match Value::deserialize(deserializer)? {
        Value::Bool(false) => Ok(Threshold::Disabled),
        Value::Bool(true) => Ok(Threshold::Default),
        Value::Number(n) => Ok(Threshold::Value(n)),
    }
}

/// Top-level configuration.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Lint configuration.
    pub lint: LintConfig,
    /// Format configuration.
    pub fmt: FmtConfig,
    /// New command configuration.
    pub new: NewConfig,
    /// Add command configuration.
    pub add: AddConfig,
}

/// Configuration for the lint command.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LintConfig {
    /// Treat warnings as errors.
    pub strict: bool,
    /// Rule-specific configuration.
    pub rules: RulesConfig,
}

/// Configuration for individual lint rules.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Enable name format validation (E001).
    pub name_format: bool,
    /// Maximum name length (E002).
    #[serde(deserialize_with = "deserialize_threshold")]
    pub name_length: Threshold,
    /// Enable name/directory match validation (E003).
    pub name_directory: bool,
    /// Require description (E004).
    pub description_required: bool,
    /// Maximum description length (E005).
    #[serde(deserialize_with = "deserialize_threshold")]
    pub description_length: Threshold,
    /// Maximum compatibility length (E006).
    #[serde(deserialize_with = "deserialize_threshold")]
    pub compatibility_length: Threshold,
    /// Validate referenced files exist (E009).
    pub references_exist: bool,
    /// Maximum body length in lines (W001).
    #[serde(deserialize_with = "deserialize_threshold")]
    pub body_length: Threshold,
    /// Check scripts are executable (W002).
    pub script_executable: bool,
    /// Check scripts have shebang (W003).
    pub script_shebang: bool,
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            name_format: true,
            name_length: Threshold::Default,
            name_directory: true,
            description_required: true,
            description_length: Threshold::Default,
            compatibility_length: Threshold::Default,
            references_exist: true,
            body_length: Threshold::Default,
            script_executable: true,
            script_shebang: true,
        }
    }
}

/// Configuration for the fmt command.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FmtConfig {
    /// Sort frontmatter keys.
    pub sort_frontmatter: bool,
    /// Indentation size.
    pub indent_size: usize,
    /// Format markdown tables.
    pub format_tables: bool,
}

impl Default for FmtConfig {
    fn default() -> Self {
        Self {
            sort_frontmatter: true,
            indent_size: 2,
            format_tables: true,
        }
    }
}

/// Configuration for the new command.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct NewConfig {
    /// Default license for new skills.
    pub default_license: Option<String>,
    /// Default template for new skills.
    pub default_template: String,
    /// Default script language for new skills.
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

/// Configuration for the add command.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AddConfig {
    /// Target agent for skill installation. If None, installs to ./skills/ in current directory.
    pub default_agent: Option<Agent>,
    /// Prompt before installing (false for CI).
    pub confirm: bool,
    /// Validate skills before installing.
    pub validate: bool,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            default_agent: None,
            confirm: true,
            validate: true,
        }
    }
}

impl Config {
    /// Load configuration from a file or find it automatically.
    pub fn load(path: Option<&PathBuf>) -> std::result::Result<Self, std::io::Error> {
        let config_path = path.cloned().or_else(Self::find_config);

        let Some(config_path) = config_path else {
            return Ok(Self::default());
        };

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
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
