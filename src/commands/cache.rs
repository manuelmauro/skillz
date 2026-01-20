//! Cache management commands.

use crate::cache::{clean_all, clean_old_checkouts, format_size, git_dir, CacheStats};
use crate::cli::{CacheArgs, CacheCommand, Cli};
use crate::config::Config;
use crate::error::SkiloError;
use colored::Colorize;
use std::time::SystemTime;

/// Run the cache command.
pub fn run(args: CacheArgs, _config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    match args.command {
        Some(CacheCommand::Path) => show_path(cli),
        Some(CacheCommand::Clean { all, max_age }) => clean(all, max_age, cli),
        None => show_status(cli),
    }
}

/// Show cache directory path.
fn show_path(_cli: &Cli) -> Result<i32, SkiloError> {
    let git = git_dir()
        .ok_or_else(|| SkiloError::Config("Could not determine cache directory".to_string()))?;

    println!("{}", git.display());

    Ok(0)
}

/// Show cache status.
fn show_status(cli: &Cli) -> Result<i32, SkiloError> {
    let git = git_dir()
        .ok_or_else(|| SkiloError::Config("Could not determine cache directory".to_string()))?;

    if !git.exists() {
        if !cli.quiet {
            println!("Cache directory: {} (not created yet)", git.display());
        }
        return Ok(0);
    }

    let stats = CacheStats::collect();

    println!("Cache directory: {}", git.display().to_string().cyan());
    println!();

    // Show db stats
    println!(
        "  {}: {} repositories, {}",
        "db/".bold(),
        stats.repos.len(),
        format_size(stats.db_size)
    );
    for repo in &stats.repos {
        println!("    {}", repo.name);
    }

    if !stats.repos.is_empty() && !stats.checkouts.is_empty() {
        println!();
    }

    // Show checkout stats
    println!(
        "  {}: {} checkouts, {}",
        "checkouts/".bold(),
        stats.checkouts.len(),
        format_size(stats.checkouts_size)
    );
    for checkout in &stats.checkouts {
        let age = format_age(checkout.modified);
        println!("    {} {}", checkout.name, age.dimmed());
    }

    if !stats.checkouts.is_empty() || !stats.repos.is_empty() {
        println!();
        println!("Total: {}", format_size(stats.total_size()).cyan());
    }

    Ok(0)
}

/// Format age as a human-readable string.
fn format_age(modified: Option<SystemTime>) -> String {
    let Some(modified) = modified else {
        return String::new();
    };

    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return String::new();
    };

    let secs = age.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    let weeks = days / 7;

    if weeks > 0 {
        format!("({} week{} ago)", weeks, if weeks == 1 { "" } else { "s" })
    } else if days > 0 {
        format!("({} day{} ago)", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("({} hour{} ago)", hours, if hours == 1 { "" } else { "s" })
    } else if mins > 0 {
        format!("({} minute{} ago)", mins, if mins == 1 { "" } else { "s" })
    } else {
        "(just now)".to_string()
    }
}

/// Clean cache.
fn clean(all: bool, max_age: u32, cli: &Cli) -> Result<i32, SkiloError> {
    if all {
        if !cli.quiet {
            println!("Removing all cached data...");
        }

        let (repos, checkouts, freed) = clean_all().map_err(SkiloError::Io)?;

        if !cli.quiet {
            println!(
                "Removed {} repositories, {} checkouts ({} freed)",
                repos,
                checkouts,
                format_size(freed).green()
            );
        }
    } else {
        if !cli.quiet {
            println!("Removing checkouts older than {} days...", max_age);
        }

        let (removed, freed) = clean_old_checkouts(max_age).map_err(SkiloError::Io)?;

        if !cli.quiet {
            if removed > 0 {
                println!(
                    "Removed {} checkout{} ({} freed)",
                    removed,
                    if removed == 1 { "" } else { "s" },
                    format_size(freed).green()
                );
            } else {
                println!("No checkouts older than {} days found", max_age);
            }
        }
    }

    Ok(0)
}
