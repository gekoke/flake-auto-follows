use clap::Parser;
use cli::{Cli, Command};
use color_eyre::{
    eyre::{Context, Result},
    owo_colors::OwoColorize,
};
use commands::{check, fix};
use nix::{resolve_flake_filepath, FlakeRef};

use crate::tracing::configure_tracing;

mod cli;
mod commands;
mod config;
mod metadata;
mod nix;
mod tracing;

fn main() -> Result<()> {
    let cli = Cli::parse();
    configure_tracing(cli.verbose)?;

    let result = match cli.command {
        Command::Check { common_args } => {
            let flake_ref = FlakeRef(common_args.flake_ref);
            let config = config::get(&common_args.config_path)?;
            check(&flake_ref, &config.rules)
        }
        Command::Fix {
            common_args,
            non_interactive,
        } => {
            let flake_ref = FlakeRef(common_args.flake_ref);

            // Ensure we can resolve to a local file before proceeding
            let _ = resolve_flake_filepath(&flake_ref)
                .context("the `fix` command needs a file to operate on")?;

            let config = config::get(&common_args.config_path)?;
            fix(&flake_ref, &config.rules, non_interactive)
        }
    };

    if result.is_ok() {
        println!("{}", "All done. Bye!".blue().italic());
    }; 
    result
}
