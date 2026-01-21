//! List detected AI coding agents.

use crate::agent::{Agent, AgentFeatures, DetectedAgent};
use crate::cli::{AgentsArgs, Cli};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use colored::Colorize;
use std::path::PathBuf;

/// Run the agents command.
///
/// Lists all detected agents at project and global levels.
pub fn run(args: AgentsArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Detect all agents
    let detected = Agent::detect_all(&project_root);

    if detected.is_empty() {
        formatter.format_message("No agents detected.");
        match config.add.default_agent {
            Some(agent) => {
                formatter.format_message(&format!(
                    "\nDefault agent: {} ({})",
                    agent.display_name(),
                    agent.skills_dir()
                ));
            }
            None => {
                formatter.format_message("\nDefault: ./skills/ (no agent configured)");
            }
        }
        return Ok(0);
    }

    // Group by agent type
    let mut project_agents: Vec<&DetectedAgent> = Vec::new();
    let mut global_agents: Vec<&DetectedAgent> = Vec::new();

    for agent in &detected {
        if agent.is_global {
            global_agents.push(agent);
        } else {
            project_agents.push(agent);
        }
    }

    // Print project agents
    if !project_agents.is_empty() {
        println!("{}", "Project agents:".bold());
        for agent in &project_agents {
            print_agent_info(agent, args.verbose);
        }
        println!();
    }

    // Print global agents
    if !global_agents.is_empty() {
        println!("{}", "Global agents:".bold());
        for agent in &global_agents {
            print_agent_info(agent, args.verbose);
        }
        println!();
    }

    // Show feature matrix if verbose
    if args.verbose {
        println!("{}", "Feature support:".bold());
        print_feature_matrix();
    }

    Ok(0)
}

/// Print information about a detected agent.
fn print_agent_info(agent: &DetectedAgent, verbose: bool) {
    let skill_text = if agent.skill_count == 1 {
        "1 skill"
    } else {
        &format!("{} skills", agent.skill_count)
    };

    println!(
        "  {:<14} {}  ({})",
        agent.agent.display_name().cyan(),
        agent.skills_path.display(),
        skill_text.dimmed()
    );

    if verbose {
        let features = agent.agent.features();
        print_features(&features);
    }
}

/// Print agent features inline.
fn print_features(features: &AgentFeatures) {
    let mut supported = Vec::new();
    if features.context_fork {
        supported.push("context:fork");
    }
    if features.hooks {
        supported.push("hooks");
    }
    if features.allowed_tools {
        supported.push("allowed-tools");
    }
    if features.scripts {
        supported.push("scripts");
    }

    if !supported.is_empty() {
        println!(
            "    {} {}",
            "Features:".dimmed(),
            supported.join(", ").dimmed()
        );
    }
}

/// Print the full feature matrix for all agents.
fn print_feature_matrix() {
    println!();
    println!(
        "  {:<14} {:^12} {:^8} {:^14} {:^8}",
        "Agent".bold(),
        "context:fork".bold(),
        "hooks".bold(),
        "allowed-tools".bold(),
        "scripts".bold()
    );
    println!("  {}", "-".repeat(60));

    for agent in Agent::all() {
        let features = agent.features();
        println!(
            "  {:<14} {:^12} {:^8} {:^14} {:^8}",
            agent.display_name(),
            feature_mark(features.context_fork),
            feature_mark(features.hooks),
            feature_mark(features.allowed_tools),
            feature_mark(features.scripts),
        );
    }
}

/// Return a mark for feature support.
fn feature_mark(supported: bool) -> &'static str {
    if supported {
        "Yes"
    } else {
        "-"
    }
}
