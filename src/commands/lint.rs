use crate::cli::{Cli, LintArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::skill::{Discovery, Manifest, ValidationResult, Validator};

pub fn run(args: LintArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);
    let strict = args.strict || config.lint.strict;

    // Find all skills
    let skill_paths = Discovery::find_skills(&args.path);

    if skill_paths.is_empty() {
        return Err(SkiloError::NoSkillsFound {
            path: args.path.display().to_string(),
        });
    }

    // Load and validate skills
    let validator = Validator::new(&config.lint);
    let mut results: Vec<(String, ValidationResult)> = Vec::new();
    let mut parse_errors = 0;

    for path in &skill_paths {
        match Manifest::parse(path.clone()) {
            Ok(manifest) => {
                let result = validator.validate(&manifest);
                results.push((path.display().to_string(), result));
            }
            Err(e) => {
                parse_errors += 1;
                formatter.format_error(&format!("{}: {}", path.display(), e));
            }
        }
    }

    // Output results
    let output = formatter.format_validation(&results);
    if !output.is_empty() {
        print!("{}", output);
    }

    // Calculate exit code
    let total_errors: usize = results.iter().map(|(_, r)| r.errors.len()).sum();
    let total_warnings: usize = results.iter().map(|(_, r)| r.warnings.len()).sum();

    let has_errors = parse_errors > 0 || total_errors > 0;
    let has_strict_warnings = strict && total_warnings > 0;

    if has_errors || has_strict_warnings {
        Ok(1)
    } else {
        Ok(0)
    }
}
