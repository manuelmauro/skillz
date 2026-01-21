//! List installed skills.

use crate::agent::Agent;
use crate::cli::{Cli, ListArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::scope::{list_skills, list_skills_from_path, InstalledSkill, Scope};
use colored::Colorize;

/// Run the list command.
///
/// Lists installed skills at project or global level.
pub fn run(args: ListArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);
    let project_root = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    // Determine agent (None means use ./skills/)
    let agent: Option<Agent> = match args.agent.as_ref().map(|a| a.to_selection()) {
        Some(crate::cli::AgentSelection::Single(a)) => Some(a),
        Some(crate::cli::AgentSelection::All) => config.add.default_agent,
        None => config.add.default_agent,
    };

    // Collect skills based on flags
    let (project_skills, global_skills) = match agent {
        Some(agent) => {
            if args.all {
                // List both project and global
                let project = list_skills(agent, Scope::Project, &project_root);
                let global = list_skills(agent, Scope::Global, &project_root);
                (project, global)
            } else if args.global {
                // List only global
                let global = list_skills(agent, Scope::Global, &project_root);
                (Vec::new(), global)
            } else {
                // List only project (default)
                let project = list_skills(agent, Scope::Project, &project_root);
                (project, Vec::new())
            }
        }
        None => {
            // No agent configured - use ./skills/
            if args.global {
                formatter.format_error("Global listing requires an agent (use --agent)");
                return Ok(1);
            }
            let skills_dir = project_root.join("skills");
            let project = list_skills_from_path(&skills_dir, None, Scope::Project);
            (project, Vec::new())
        }
    };

    let total_skills = project_skills.len() + global_skills.len();

    if total_skills == 0 {
        let scope_desc = if args.all {
            "at project or global level"
        } else if args.global {
            "globally"
        } else {
            "in project"
        };
        let target = agent
            .map(|a| a.display_name().to_string())
            .unwrap_or_else(|| "skills/".to_string());
        formatter.format_message(&format!(
            "No skills installed {} for {}.",
            scope_desc, target
        ));
        return Ok(0);
    }

    // Print project skills
    if !project_skills.is_empty() {
        let dir = agent
            .map(|a| a.skills_dir().to_string())
            .unwrap_or_else(|| "skills/".to_string());
        println!("{} ({}):", "Project skills".bold(), dir.dimmed());
        print_skills(&project_skills);

        if !global_skills.is_empty() {
            println!();
        }
    }

    // Print global skills
    if !global_skills.is_empty() {
        if let Some(agent) = agent {
            let dir = agent.global_skills_dir();
            println!("{} ({}):", "Global skills".bold(), dir.dimmed());
            print_skills(&global_skills);
        }
    }

    // Check for shadowed skills
    if args.all && !project_skills.is_empty() && !global_skills.is_empty() {
        let project_names: std::collections::HashSet<_> =
            project_skills.iter().map(|s| &s.name).collect();

        let shadowed: Vec<_> = global_skills
            .iter()
            .filter(|s| project_names.contains(&s.name))
            .collect();

        if !shadowed.is_empty() {
            println!();
            println!(
                "{}: {} global skill(s) shadowed by project skills:",
                "Note".yellow(),
                shadowed.len()
            );
            for skill in &shadowed {
                println!("  {} {}", "-".dimmed(), skill.name.dimmed());
            }
        }
    }

    Ok(0)
}

/// Print a list of skills.
fn print_skills(skills: &[InstalledSkill]) {
    let max_name_len = skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(20)
        .max(10);

    for skill in skills {
        let description = truncate_description(&skill.description, 50);
        println!(
            "  {:<width$}  {}",
            skill.name.cyan(),
            description,
            width = max_name_len
        );
    }
}

/// Truncate a description to a maximum length, adding ellipsis if needed.
fn truncate_description(s: &str, max_len: usize) -> String {
    if s.is_empty() {
        return "(no description)".dimmed().to_string();
    }

    let first_sentence = s.split(". ").next().unwrap_or(s);

    if first_sentence.len() <= max_len {
        first_sentence.to_string()
    } else {
        format!("{}...", &first_sentence[..max_len.saturating_sub(3)])
    }
}
