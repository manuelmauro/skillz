//! Creates new skills from templates.

use crate::cli::{Cli, NewArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::scope::{ensure_skills_dir, Scope};
use crate::templates::{get_template, TemplateContext};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

/// Pattern for valid skill names.
static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

/// Run the new command.
///
/// Creates a new skill from the specified template.
pub fn run(args: NewArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);

    // Validate name
    if !NAME_REGEX.is_match(&args.name) {
        return Err(SkiloError::InvalidName(args.name));
    }

    if args.name.len() > 64 {
        return Err(SkiloError::InvalidName(format!(
            "{} (name too long, max 64 chars)",
            args.name
        )));
    }

    // Determine output directory based on --output, --agent, --global flags
    let output_dir = resolve_output_dir(&args, config)?;
    let skill_dir = output_dir.join(&args.name);

    // Check if skill already exists
    if skill_dir.exists() {
        return Err(SkiloError::SkillExists {
            name: args.name,
            path: skill_dir.display().to_string(),
        });
    }

    // Get license (from args or config)
    let license = args.license.or_else(|| config.new.default_license.clone());

    // Build template context
    let ctx = TemplateContext {
        name: args.name.clone(),
        description: args
            .description
            .unwrap_or_else(|| format!("A {} skill.", args.name.replace('-', " "))),
        license,
        lang: args.lang,
        include_optional_dirs: !args.no_optional_dirs,
        include_scripts: !args.no_scripts,
    };

    // Render template
    let template = get_template(args.template);
    template.render(&ctx, &output_dir)?;

    formatter.format_success(&format!(
        "Created skill '{}' at {}",
        args.name,
        skill_dir.display()
    ));

    Ok(0)
}

/// Resolve the output directory based on CLI arguments.
fn resolve_output_dir(args: &NewArgs, config: &Config) -> Result<PathBuf, SkiloError> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // If --output is specified, use it directly
    if let Some(ref output) = args.output {
        return Ok(output.clone());
    }

    // Determine scope
    let scope = if args.global {
        Scope::Global
    } else {
        Scope::Project
    };

    // Determine agent
    let agent: Option<crate::agent::Agent> = if let Some(ref cli_agent) = args.agent {
        match cli_agent.to_selection() {
            crate::cli::AgentSelection::Single(a) => Some(a),
            crate::cli::AgentSelection::All => config.add.default_agent,
        }
    } else {
        config.add.default_agent
    };

    // Ensure skills directory exists and return it
    match agent {
        Some(agent) => ensure_skills_dir(agent, scope, &project_root)
            .map_err(|e| SkiloError::Config(format!("Failed to create skills directory: {}", e))),
        None => {
            if args.global {
                return Err(SkiloError::Config(
                    "Global installation requires an agent (use --agent)".to_string(),
                ));
            }
            let skills_dir = project_root.join("skills");
            std::fs::create_dir_all(&skills_dir).map_err(|e| {
                SkiloError::Config(format!("Failed to create skills directory: {}", e))
            })?;
            Ok(skills_dir)
        }
    }
}
