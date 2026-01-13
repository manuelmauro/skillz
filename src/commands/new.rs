use crate::cli::{Cli, NewArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::templates::{get_template, TemplateContext};
use once_cell::sync::Lazy;
use regex::Regex;

static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

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

    // Determine output directory
    let output_dir = args
        .output
        .unwrap_or_else(|| std::env::current_dir().unwrap());
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
