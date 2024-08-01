use std::ffi::OsString;

use clap::{Args, Parser, Subcommand};

/// Automatically follow Nix flake inputs
#[derive(Parser)]
#[command(name = "faf")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Whether to enable logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args)]
pub struct CommonArgs {
    /// A url-like reference to the flake
    #[arg(short, long = "flake")]
    pub flake_ref: String,

    /// A path to the program configuration
    #[arg(required = true, short, long = "config", default_value = "./faf.toml")]
    pub config_path: OsString,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check for missing follows in flake inputs
    Check {
        #[command(flatten)]
        common_args: CommonArgs,
    },
    /// Fix missing follows in flake inputs
    Fix {
        #[command(flatten)]
        common_args: CommonArgs,

        /// Whether to run the command non-interactively (not prompt the user)
        #[arg(short, long)]
        non_interactive: bool,
    },
}
