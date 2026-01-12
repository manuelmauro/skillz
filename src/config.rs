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
