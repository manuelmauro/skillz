use crate::cli::{Cli, FmtArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;
use crate::skill::{Discovery, Formatter, FormatterConfig, Manifest};
use colored::Colorize;

pub fn run(args: FmtArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let output_formatter = get_formatter(cli.format, cli.quiet);
    let skill_formatter = Formatter::new(FormatterConfig::from(&config.fmt));

    // Find all skills
    let skill_paths = Discovery::find_skills(&args.path);

    if skill_paths.is_empty() {
        return Err(SkiloError::NoSkillsFound {
            path: args.path.display().to_string(),
        });
    }

    let mut files_changed = 0;
    let mut files_checked = 0;

    for path in &skill_paths {
        match Manifest::parse(path.clone()) {
            Ok(manifest) => {
                files_checked += 1;

                // Get formatted content
                let formatted = match skill_formatter.format(&manifest) {
                    Ok(f) => f,
                    Err(e) => {
                        output_formatter.format_error(&format!("{}: {}", path.display(), e));
                        continue;
                    }
                };

                // Read current content
                let current = std::fs::read_to_string(path)?;

                if formatted != current {
                    files_changed += 1;

                    if args.check {
                        output_formatter.format_message(&format!(
                            "{} {} needs formatting",
                            "!".yellow(),
                            path.display()
                        ));
                    } else if args.diff {
                        // Show diff
                        println!("{}", format!("--- {}", path.display()).dimmed());
                        println!("{}", format!("+++ {}", path.display()).dimmed());
                        print_diff(&current, &formatted);
                    } else {
                        // Write formatted content
                        std::fs::write(path, &formatted)?;
                        output_formatter.format_message(&format!(
                            "{} Formatted {}",
                            "✓".green(),
                            path.display()
                        ));
                    }
                }
            }
            Err(e) => {
                output_formatter.format_error(&format!("{}: {}", path.display(), e));
            }
        }
    }

    if args.check {
        if files_changed > 0 {
            output_formatter.format_message(&format!(
                "\n{} {} file(s) need formatting",
                "!".yellow(),
                files_changed
            ));
            Ok(1)
        } else {
            output_formatter.format_success(&format!(
                "{} file(s) checked, all formatted correctly",
                files_checked
            ));
            Ok(0)
        }
    } else {
        if files_changed > 0 {
            output_formatter.format_success(&format!("Formatted {} file(s)", files_changed));
        } else {
            output_formatter.format_success(&format!(
                "{} file(s) already formatted correctly",
                files_checked
            ));
        }
        Ok(0)
    }
}

fn print_diff(old: &str, new: &str) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple line-by-line diff
    let max_lines = old_lines.len().max(new_lines.len());

    for i in 0..max_lines {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(i).copied();

        match (old_line, new_line) {
            (Some(o), Some(n)) if o == n => {
                println!(" {}", o);
            }
            (Some(o), Some(n)) => {
                println!("{}", format!("-{}", o).red());
                println!("{}", format!("+{}", n).green());
            }
            (Some(o), None) => {
                println!("{}", format!("-{}", o).red());
            }
            (None, Some(n)) => {
                println!("{}", format!("+{}", n).green());
            }
            (None, None) => {}
        }
    }
}
