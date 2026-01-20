//! Skilo CLI binary.

use clap::Parser;
use miette::Result;
use skilo::cli::{Cli, Command};
use skilo::commands;
use skilo::config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load(cli.config.as_ref())
        .map_err(|e| miette::miette!("Failed to load config: {}", e))?;

    let exit_code = match &cli.command {
        Command::Add(args) => commands::add::run(args.clone(), &config, &cli)?,
        Command::New(args) => commands::new::run(args.clone(), &config, &cli)?,
        Command::Lint(args) => commands::lint::run(args.clone(), &config, &cli)?,
        Command::Fmt(args) => commands::fmt::run(args.clone(), &config, &cli)?,
        Command::Check(args) => commands::check::run(args.clone(), &config, &cli)?,
        Command::Validate(args) => {
            let mut args = args.clone();
            args.strict = true;
            commands::lint::run(args, &config, &cli)?
        }
        Command::ReadProperties(args) => {
            commands::read_properties::run(args.clone(), &config, &cli)?
        }
        Command::ToPrompt(args) => commands::to_prompt::run(args.clone(), &config, &cli)?,
        Command::List(args) => commands::list::run(args.clone(), &config, &cli)?,
        Command::Agents(args) => commands::agents::run(args.clone(), &config, &cli)?,
        Command::Cache(args) => commands::cache::run(args.clone(), &config, &cli)?,
    };

    std::process::exit(exit_code);
}
