use crate::cli::{CheckArgs, Cli, FmtArgs, LintArgs};
use crate::config::Config;
use crate::error::SkiloError;
use crate::output::get_formatter;

pub fn run(args: CheckArgs, config: &Config, cli: &Cli) -> Result<i32, SkiloError> {
    let formatter = get_formatter(cli.format, cli.quiet);

    formatter.format_message("Running lint...");

    // Run lint with strict mode
    let lint_args = LintArgs {
        path: args.path.clone(),
        strict: true,
        fix: false,
    };
    let lint_result = super::lint::run(lint_args, config, cli)?;

    formatter.format_message("\nRunning format check...");

    // Run format check
    let fmt_args = FmtArgs {
        path: args.path,
        check: true,
        diff: false,
    };
    let fmt_result = super::fmt::run(fmt_args, config, cli)?;

    // Return non-zero if either failed
    if lint_result != 0 || fmt_result != 0 {
        Ok(1)
    } else {
        formatter.format_success("\nAll checks passed!");
        Ok(0)
    }
}
