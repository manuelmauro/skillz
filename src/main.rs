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
        Command::New(args) => commands::new::run(args.clone(), &config, &cli)?,
        Command::Lint(args) => commands::lint::run(args.clone(), &config, &cli)?,
        Command::Fmt(args) => commands::fmt::run(args.clone(), &config, &cli)?,
        Command::Check(args) => commands::check::run(args.clone(), &config, &cli)?,
        Command::Validate(args) => {
            let mut args = args.clone();
            args.strict = true;
            commands::lint::run(args, &config, &cli)?
        }
    };

    std::process::exit(exit_code);
}
