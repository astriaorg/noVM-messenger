use clap::Subcommand;
use color_eyre::eyre;
mod send_text;
mod submit;
mod transfer;

#[derive(Debug, clap::Args)]
pub(super) struct Command {
    #[command(subcommand)]
    command: SubCommand,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        match self.command {
            SubCommand::Transfer(transfer) => transfer.run().await,
            SubCommand::Text(send_text) => send_text.run().await,
            // SubCommand::Submit(submit) => submit.run().await,
        }
    }
}

/// Interact with a Sequencer node
#[derive(Debug, Subcommand)]
enum SubCommand {
    Transfer(transfer::Command),
    Text(send_text::Command),
}
