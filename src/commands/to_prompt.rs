use crate::cli::{Cli, ToPromptArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::skill::{Discovery, Manifest};
use serde::Serialize;
use std::path::PathBuf;

/// Root element for XML output
#[derive(Serialize)]
#[serde(rename = "available_skills")]
struct AvailableSkills {
    #[serde(rename = "skill")]
    skills: Vec<SkillEntry>,
}

/// Represents a skill for XML output
#[derive(Serialize)]
struct SkillEntry {
    name: String,
    description: String,
    location: String,
}

impl From<&Manifest> for SkillEntry {
    fn from(manifest: &Manifest) -> Self {
        Self {
            name: manifest.frontmatter.name.clone(),
            description: manifest.frontmatter.description.clone(),
            location: manifest.path.display().to_string(),
        }
    }
}

pub fn run(args: ToPromptArgs, _config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
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

    // Parse all skills and collect entries
    let mut skills: Vec<SkillEntry> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for path in &all_skill_paths {
        match Manifest::parse(path.clone()) {
            Ok(manifest) => {
                skills.push(SkillEntry::from(&manifest));
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

    // Generate and output XML
    if !cli.quiet {
        let available_skills = AvailableSkills { skills };
        let mut buffer = String::new();
        let mut serializer = quick_xml::se::Serializer::new(&mut buffer);
        serializer.indent(' ', 2);
        available_skills
            .serialize(serializer)
            .map_err(|e| SkiloError::Config(format!("XML serialization failed: {}", e)))?;
        println!("{}", buffer);
    }

    // Return error code if there were parsing failures
    if errors.is_empty() {
        Ok(0)
    } else {
        Ok(1)
    }
}
