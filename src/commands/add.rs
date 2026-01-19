//! The `add` command implementation for installing skills from git repositories.

use crate::cli::{AddArgs, Cli};
use crate::config::Config;
use crate::git::{fetch, Source};
use crate::output::get_formatter;
use crate::skill::discovery::Discovery;
use crate::skill::manifest::Manifest;
use crate::skill::validator::Validator;
use crate::SkiloError;
use colored::Colorize;
use dialoguer::Confirm;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Information about a discovered skill.
#[derive(Clone)]
struct SkillInfo {
    /// The name of the skill.
    name: String,
    /// The description of the skill.
    description: String,
    /// The source path (within the fetched repo).
    source_path: PathBuf,
    /// Whether the skill passed validation.
    valid: bool,
    /// Validation errors, if any.
    errors: Vec<String>,
}

/// Run the add command.
pub fn run(args: AddArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    use crate::agent::Agent;

    let formatter = get_formatter(cli.format, cli.quiet);

    // Determine install directory: --output > --agent > config default
    let install_dir = if let Some(ref output) = args.output {
        output.clone()
    } else if let Some(cli_agent) = args.agent {
        let agent: Agent = cli_agent.into();
        PathBuf::from(agent.skills_dir())
    } else {
        PathBuf::from(config.add.default_agent.skills_dir())
    };

    // Parse the source
    let source = Source::parse_with_options(&args.source, args.branch.clone(), args.tag.clone())?;

    match source {
        Source::Git(git_source) => {
            let display_name = git_source.display_name();

            if !cli.quiet {
                print!("Fetching skills from {}...", display_name.cyan());
                io::stdout().flush().ok();
            }

            // Fetch the repository
            let fetch_result = fetch(&git_source)?;

            if !cli.quiet {
                println!(" {}", "done".green());
            }

            // Discover skills in the fetched repository
            let skills = discover_skills(&fetch_result.root, config)?;

            if skills.is_empty() {
                return Err(SkiloError::NoSkillsFound { path: display_name });
            }

            // Filter by --skill if provided
            let skills = filter_skills(skills, &args.skill);

            if skills.is_empty() {
                formatter.format_error(&format!(
                    "No skills found matching: {}",
                    args.skill
                        .as_ref()
                        .map(|v| v.join(", "))
                        .unwrap_or_default()
                ));
                return Ok(1);
            }

            // List mode
            if args.list {
                print_skill_list(&skills);
                return Ok(0);
            }

            // Confirm installation
            let confirm = args.yes || !config.add.confirm;
            if !confirm {
                print_skill_list(&skills);
                println!();

                let prompt = format!(
                    "Install {} skill{} to {}/?",
                    skills.len(),
                    if skills.len() == 1 { "" } else { "s" },
                    install_dir.display()
                );

                if !Confirm::new()
                    .with_prompt(prompt)
                    .default(false)
                    .interact()
                    .map_err(|_| SkiloError::Cancelled)?
                {
                    return Err(SkiloError::Cancelled);
                }

                println!();
            }

            // Install skills
            let installed = install_skills(&skills, &install_dir, args.yes, cli.quiet)?;

            if !cli.quiet {
                println!();
                formatter.format_success(&format!(
                    "Installed {} skill{} to {}/",
                    installed,
                    if installed == 1 { "" } else { "s" },
                    install_dir.display()
                ));
            }

            if installed == 0 {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Source::Local(path) => {
            // Discover skills in the local path
            let skills = discover_skills(&path, config)?;

            if skills.is_empty() {
                return Err(SkiloError::NoSkillsFound {
                    path: path.display().to_string(),
                });
            }

            // Filter by --skill if provided
            let skills = filter_skills(skills, &args.skill);

            if skills.is_empty() {
                formatter.format_error(&format!(
                    "No skills found matching: {}",
                    args.skill
                        .as_ref()
                        .map(|v| v.join(", "))
                        .unwrap_or_default()
                ));
                return Ok(1);
            }

            // List mode
            if args.list {
                print_skill_list(&skills);
                return Ok(0);
            }

            // Confirm installation
            let confirm = args.yes || !config.add.confirm;
            if !confirm {
                print_skill_list(&skills);
                println!();

                let prompt = format!(
                    "Install {} skill{} to {}/?",
                    skills.len(),
                    if skills.len() == 1 { "" } else { "s" },
                    install_dir.display()
                );

                if !Confirm::new()
                    .with_prompt(prompt)
                    .default(false)
                    .interact()
                    .map_err(|_| SkiloError::Cancelled)?
                {
                    return Err(SkiloError::Cancelled);
                }

                println!();
            }

            // Install skills
            let installed = install_skills(&skills, &install_dir, args.yes, cli.quiet)?;

            if !cli.quiet {
                println!();
                formatter.format_success(&format!(
                    "Installed {} skill{} to {}/",
                    installed,
                    if installed == 1 { "" } else { "s" },
                    install_dir.display()
                ));
            }

            if installed == 0 {
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }
}

/// Discover skills in a directory.
fn discover_skills(root: &Path, config: &Config) -> Result<Vec<SkillInfo>, SkiloError> {
    use crate::agent::Agent;
    use std::collections::HashSet;

    let mut skills = Vec::new();
    let mut seen_paths = HashSet::new();

    // Use the existing discovery mechanism
    let skill_paths = Discovery::find_skills(root);

    if skill_paths.is_empty() {
        // Try looking in common locations and all agent-specific directories
        let mut locations: Vec<&str> = vec!["skills"];
        locations.extend(Agent::all().iter().map(|a| a.skills_dir()));

        for loc in locations {
            let path = root.join(loc);
            if path.exists() {
                let found = Discovery::find_skills(&path);
                for skill_path in found {
                    if seen_paths.insert(skill_path.clone()) {
                        if let Some(info) = load_skill_info(&skill_path, config) {
                            skills.push(info);
                        }
                    }
                }
            }
        }
    } else {
        for skill_path in skill_paths {
            if seen_paths.insert(skill_path.clone()) {
                if let Some(info) = load_skill_info(&skill_path, config) {
                    skills.push(info);
                }
            }
        }
    }

    // Sort by name
    skills.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(skills)
}

/// Load skill info from a SKILL.md path.
fn load_skill_info(skill_path: &Path, config: &Config) -> Option<SkillInfo> {
    let manifest = match Manifest::parse(skill_path.to_path_buf()) {
        Ok(m) => m,
        Err(_) => return None,
    };

    // Validate the skill
    let validator = Validator::new(&config.lint);
    let result = validator.validate(&manifest);

    let valid = result.errors.is_empty();
    let errors: Vec<String> = result.errors.iter().map(|d| d.message.clone()).collect();

    // Get the skill directory (parent of SKILL.md)
    let source_path = skill_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| skill_path.to_path_buf());

    Some(SkillInfo {
        name: manifest.frontmatter.name.clone(),
        description: manifest.frontmatter.description.clone(),
        source_path,
        valid,
        errors,
    })
}

/// Filter skills by name.
fn filter_skills(skills: Vec<SkillInfo>, filter: &Option<Vec<String>>) -> Vec<SkillInfo> {
    match filter {
        Some(names) => skills
            .into_iter()
            .filter(|s| names.iter().any(|n| n == &s.name))
            .collect(),
        None => skills,
    }
}

/// Print the list of discovered skills.
fn print_skill_list(skills: &[SkillInfo]) {
    println!();
    println!(
        "Found {} skill{}:",
        skills.len(),
        if skills.len() == 1 { "" } else { "s" }
    );

    let max_name_len = skills.iter().map(|s| s.name.len()).max().unwrap_or(20);

    for skill in skills {
        let status = if skill.valid {
            "".to_string()
        } else {
            format!(" {}", "(invalid)".yellow())
        };

        let description = truncate_description(&skill.description, 50);

        println!(
            "  {:<width$}  {}{}",
            skill.name.cyan(),
            description,
            status,
            width = max_name_len
        );
    }
}

/// Truncate a description to a maximum length, adding ellipsis if needed.
fn truncate_description(s: &str, max_len: usize) -> String {
    // Take first sentence or truncate
    let first_sentence = s.split(". ").next().unwrap_or(s);

    if first_sentence.len() <= max_len {
        first_sentence.to_string()
    } else {
        format!("{}...", &first_sentence[..max_len.saturating_sub(3)])
    }
}

/// Install skills to the target directory.
fn install_skills(
    skills: &[SkillInfo],
    install_dir: &Path,
    skip_confirm: bool,
    quiet: bool,
) -> Result<usize, SkiloError> {
    // Create the install directory if needed
    fs::create_dir_all(install_dir)?;

    let mut installed = 0;

    for skill in skills {
        if !skill.valid {
            if !quiet {
                println!(
                    "Skipping {} (validation failed: {})",
                    skill.name.yellow(),
                    skill.errors.join(", ")
                );
            }
            continue;
        }

        let dest = install_dir.join(&skill.name);

        // Check if already exists
        if dest.exists() {
            if skip_confirm {
                // Overwrite silently in --yes mode
                fs::remove_dir_all(&dest)?;
            } else {
                let prompt = format!("Skill '{}' already exists. Overwrite?", skill.name);
                if !Confirm::new()
                    .with_prompt(prompt)
                    .default(false)
                    .interact()
                    .map_err(|_| SkiloError::Cancelled)?
                {
                    if !quiet {
                        println!("Skipping {}...", skill.name);
                    }
                    continue;
                }
                fs::remove_dir_all(&dest)?;
            }
        }

        if !quiet {
            print!("Installing {}...", skill.name.cyan());
            io::stdout().flush().ok();
        }

        // Copy the skill directory
        copy_dir_all(&skill.source_path, &dest)?;

        if !quiet {
            println!(" {}", "done".green());
        }

        installed += 1;
    }

    Ok(installed)
}

/// Recursively copy a directory.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), SkiloError> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_skills_empty() {
        let temp = TempDir::new().unwrap();
        let config = Config::default();

        let skills = discover_skills(temp.path(), &config).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_filter_skills() {
        let skills = vec![
            SkillInfo {
                name: "skill-a".to_string(),
                description: "Skill A".to_string(),
                source_path: PathBuf::from("/tmp/a"),
                valid: true,
                errors: vec![],
            },
            SkillInfo {
                name: "skill-b".to_string(),
                description: "Skill B".to_string(),
                source_path: PathBuf::from("/tmp/b"),
                valid: true,
                errors: vec![],
            },
        ];

        let filtered = filter_skills(skills.clone(), &Some(vec!["skill-a".to_string()]));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "skill-a");

        let filtered = filter_skills(skills, &None);
        assert_eq!(filtered.len(), 2);
    }
}
