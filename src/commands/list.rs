//! List installed skills.

use crate::agent::Agent;
use crate::cli::{AgentSelection, Cli, ListArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::scope::{list_skills, InstalledSkill, Scope};
use colored::Colorize;

/// Run the list command.
///
/// Lists installed skills at project or global level.
pub fn run(args: ListArgs, _config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);
    let project_root = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    // Determine agent selection (default to "all" when no agent specified)
    let selection = args
        .agent
        .as_ref()
        .map(|a| a.to_selection())
        .unwrap_or(AgentSelection::All);

    // Handle --agent all (or default): iterate over all detected agents
    if matches!(selection, AgentSelection::All) {
        return run_for_all_agents(&args, &project_root, formatter.as_ref());
    }

    // Single agent specified
    let agent: Agent = match selection {
        AgentSelection::Single(a) => a,
        AgentSelection::All => unreachable!(), // handled above
    };

    // Collect skills based on flags (specific agent was requested)
    let (project_skills, global_skills) = if args.all {
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
        formatter.format_message(&format!(
            "No skills installed {} for {}.",
            scope_desc,
            agent.display_name()
        ));
        return Ok(0);
    }

    // Print project skills
    if !project_skills.is_empty() {
        println!(
            "{} ({}):",
            "Project skills".bold(),
            agent.skills_dir().dimmed()
        );
        print_skills(&project_skills);

        if !global_skills.is_empty() {
            println!();
        }
    }

    // Print global skills
    if !global_skills.is_empty() {
        println!(
            "{} ({}):",
            "Global skills".bold(),
            agent.global_skills_dir().dimmed()
        );
        print_skills(&global_skills);
    }

    // Check for shadowed skills
    if args.all && !project_skills.is_empty() && !global_skills.is_empty() {
        print_shadowed_skills(&project_skills, &global_skills);
    }

    Ok(0)
}

/// Run the list command for all detected agents.
fn run_for_all_agents(
    args: &ListArgs,
    project_root: &std::path::Path,
    formatter: &dyn crate::output::OutputFormatter,
) -> Result<i32, SkiloError> {
    let detected = Agent::detect_all(project_root);

    if detected.is_empty() {
        formatter.format_message("No agents detected with installed skills.");
        return Ok(0);
    }

    let mut total_skills = 0;
    let mut first = true;

    // Group by scope if --all flag is set, otherwise filter by scope
    let show_project = !args.global;
    let show_global = args.global || args.all;

    // Collect and display project-level skills
    if show_project {
        let project_agents: Vec<_> = detected.iter().filter(|d| !d.is_global).collect();
        if !project_agents.is_empty() {
            // Collect skills first to check if any exist
            let mut project_skills_by_agent = Vec::new();
            for detected_agent in &project_agents {
                let skills = list_skills(detected_agent.agent, Scope::Project, project_root);
                if !skills.is_empty() {
                    project_skills_by_agent.push((detected_agent, skills));
                }
            }

            if !project_skills_by_agent.is_empty() {
                if !first {
                    println!();
                }
                println!("{}", "Project skills:".bold());
                for (detected_agent, skills) in project_skills_by_agent {
                    println!(
                        "  {} ({}):",
                        detected_agent.agent.display_name().cyan(),
                        detected_agent.agent.skills_dir().dimmed()
                    );
                    for skill in &skills {
                        let description = truncate_description(&skill.description, 50);
                        println!("    {}  {}", skill.name.cyan(), description);
                    }
                    total_skills += skills.len();
                }
                first = false;
            }
        }
    }

    // Collect and display global-level skills
    if show_global {
        let global_agents: Vec<_> = detected.iter().filter(|d| d.is_global).collect();
        if !global_agents.is_empty() {
            // Collect skills first to check if any exist
            let mut global_skills_by_agent = Vec::new();
            for detected_agent in &global_agents {
                let skills = list_skills(detected_agent.agent, Scope::Global, project_root);
                if !skills.is_empty() {
                    global_skills_by_agent.push((detected_agent, skills));
                }
            }

            if !global_skills_by_agent.is_empty() {
                if !first {
                    println!();
                }
                println!("{}", "Global skills:".bold());
                for (detected_agent, skills) in global_skills_by_agent {
                    println!(
                        "  {} ({}):",
                        detected_agent.agent.display_name().cyan(),
                        detected_agent.agent.global_skills_dir().dimmed()
                    );
                    for skill in &skills {
                        let description = truncate_description(&skill.description, 50);
                        println!("    {}  {}", skill.name.cyan(), description);
                    }
                    total_skills += skills.len();
                }
            }
        }
    }

    if total_skills == 0 {
        let scope_desc = if args.all {
            "at project or global level"
        } else if args.global {
            "globally"
        } else {
            "in project"
        };
        formatter.format_message(&format!(
            "No skills installed {} for any detected agent.",
            scope_desc
        ));
    }

    Ok(0)
}

/// Print shadowed skills warning.
fn print_shadowed_skills(project_skills: &[InstalledSkill], global_skills: &[InstalledSkill]) {
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
