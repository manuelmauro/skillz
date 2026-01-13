use crate::cli::{Cli, ReadPropertiesArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::skill::{Discovery, Manifest};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// JSON output structure for a single skill's properties
#[derive(Serialize)]
pub struct SkillProperties {
    /// Name of the skill
    pub name: String,

    /// Description of the skill
    pub description: String,

    /// License (SPDX identifier or file reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Compatibility requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Additional metadata key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Pre-approved tools (space-delimited string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,

    /// Path to the SKILL.md file
    pub path: PathBuf,
}

impl From<&Manifest> for SkillProperties {
    fn from(manifest: &Manifest) -> Self {
        Self {
            name: manifest.frontmatter.name.clone(),
            description: manifest.frontmatter.description.clone(),
            license: manifest.frontmatter.license.clone(),
            compatibility: manifest.frontmatter.compatibility.clone(),
            metadata: manifest.frontmatter.metadata.clone(),
            allowed_tools: manifest.frontmatter.allowed_tools.clone(),
            path: manifest.path.clone(),
        }
    }
}

pub fn run(args: ReadPropertiesArgs, _config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    // Collect all skill paths from all input paths
    let mut all_skill_paths: Vec<PathBuf> = Vec::new();

    for path in &args.paths {
        let paths = Discovery::find_skills(path);
        all_skill_paths.extend(paths);
    }

    if all_skill_paths.is_empty() {
        return Err(SkiloError::NoSkillsFound {
            path: args
                .paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    // Parse all skills and collect properties
    let mut properties: Vec<SkillProperties> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for path in &all_skill_paths {
        match Manifest::parse(path.clone()) {
            Ok(manifest) => {
                properties.push(SkillProperties::from(&manifest));
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    // Output errors to stderr if any
    for error in &errors {
        eprintln!("Error: {}", error);
    }

    // Output JSON (always JSON for this command, ignoring --format)
    let output = if properties.len() == 1 {
        // Single skill: output object directly
        serde_json::to_string_pretty(&properties[0])
    } else {
        // Multiple skills: output array
        serde_json::to_string_pretty(&properties)
    };

    match output {
        Ok(json) => {
            if !cli.quiet {
                println!("{}", json);
            }
        }
        Err(e) => {
            return Err(SkiloError::Config(format!(
                "JSON serialization failed: {}",
                e
            )));
        }
    }

    // Return error code if there were parsing failures
    if errors.is_empty() {
        Ok(0)
    } else {
        Ok(1)
    }
}
