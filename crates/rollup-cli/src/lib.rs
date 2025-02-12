#![allow(
    clippy::large_enum_variant,
    reason = "the CLI contains enums with diverging variants. These are oneshot types that
              are not expected to be copied, cloned, or passed around. Therefore large differences \
              between enum variants are not expected to cause performance issues."
)]
mod query;
mod submit;
mod utils;

use clap::{Parser, Subcommand};
use color_eyre::eyre;

const DEFAULT_SEQUENCER_RPC: &str = "https://rest.astria.localdev.me";
const DEFAULT_SEQUENCER_CHAIN_ID: &str = "astria-chat";

/// Run commands against the Astria network.
#[derive(Debug, Parser)]
#[command(name = "rollup-cli", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    /// Runs the Astria CLI.
    ///
    /// This is the only entry point into the Astria CLI.
    ///
    /// # Errors
    ///
    /// Returns various errors if executing a subcommand fails. The errors are
    /// not explicitly listed here.
    pub async fn run() -> eyre::Result<()> {
        let cli = Self::parse();
        match cli.command {
            Command::Submit(submit) => submit.run().await,
            Command::Query(query) => query.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Interact with rollup
    Submit(submit::Command),
    Query(query::Command),
}
