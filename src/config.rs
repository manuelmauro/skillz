use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

/// A configurable threshold that can be default, disabled, or a specific value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Threshold {
    /// Use the default value for this rule
    #[default]
    Default,
    /// Rule is disabled
    Disabled,
    /// Rule is enabled with a specific value
    Value(usize),
}

impl Threshold {
    /// Resolve the threshold to an Option<usize> given a default value.
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

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lint: LintConfig,
    pub fmt: FmtConfig,
    pub new: NewConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LintConfig {
    pub strict: bool,
    pub rules: RulesConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    pub name_format: bool,
    #[serde(deserialize_with = "deserialize_threshold")]
    pub name_length: Threshold,
    pub name_directory: bool,
    pub description_required: bool,
    #[serde(deserialize_with = "deserialize_threshold")]
    pub description_length: Threshold,
    #[serde(deserialize_with = "deserialize_threshold")]
    pub compatibility_length: Threshold,
    pub references_exist: bool,
    #[serde(deserialize_with = "deserialize_threshold")]
    pub body_length: Threshold,
    pub script_executable: bool,
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

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FmtConfig {
    pub sort_frontmatter: bool,
    pub indent_size: usize,
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
