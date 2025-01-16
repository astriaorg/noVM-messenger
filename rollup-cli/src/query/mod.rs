use clap::Subcommand;
use color_eyre::eyre;
mod balance;
mod nonce;
mod texts;

#[derive(Debug, clap::Args)]
pub(super) struct Command {
    #[command(subcommand)]
    command: SubCommand,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        match self.command {
            SubCommand::Balance(balance) => balance.run().await,
            SubCommand::Nonce(nonce) => nonce.run().await,
            SubCommand::Texts(texts) => texts.run().await,
            // SubCommand::Submit(submit) => submit.run().await,
        }
    }
}

/// Interact with a Sequencer node
#[derive(Debug, Subcommand)]
enum SubCommand {
    Balance(balance::Command),
    Nonce(nonce::Command),
    Texts(texts::Command),
}
