use astria_core::primitive::v1::asset;
use rollup_core::transaction::v1::{action::SendText, Action};

use color_eyre::eyre::{self, WrapErr as _};

use crate::utils::submit_transaction;

#[derive(clap::Args, Debug)]
pub(super) struct Command {
    /// The text message sent
    text: String,
    /// The bech32m prefix that will be used for constructing addresses using the private key
    #[arg(long, default_value = "astria")]
    prefix: String,
    /// The private key of account being sent from
    #[arg(long, env = "SEQUENCER_PRIVATE_KEY")]
    // TODO: https://github.com/astriaorg/astria/issues/594
    // Don't use a plain text private, prefer wrapper like from
    // the secrecy crate with specialized `Debug` and `Drop` implementations
    // that overwrite the key on drop and don't reveal it when printing.
    private_key: String,
    /// The url of the Sequencer node
    #[arg(
        long,
        env = "SEQUENCER_URL",
        default_value = crate::DEFAULT_SEQUENCER_RPC
    )]
    sequencer_url: String,
    /// The chain id of the sequencing chain being used
    #[arg(
        long = "sequencer.chain-id",
        env = "ROLLUP_SEQUENCER_CHAIN_ID",
        default_value = crate::DEFAULT_SEQUENCER_CHAIN_ID
    )]
    sequencer_chain_id: String,
    /// The asset to pay the transfer fees with.
    #[arg(long, default_value = "nria")]
    fee_asset: asset::Denom,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        let res = submit_transaction(
            self.sequencer_url.as_str(),
            self.sequencer_chain_id.clone(),
            &self.prefix,
            self.private_key.as_str(),
            Action::Text(SendText {
                text: self.text.clone(),
                fee_asset: self.fee_asset,
            }),
        )
        .await
        .wrap_err("failed to submit transfer transaction")?;
        if res.status().is_success() {
            println!("Transfer completed!");
            Ok(())
        } else {
            println!("Transfer failed: {:?}", res.error_for_status());
            Err(eyre::eyre!("Transfer failed"))
        }
    }
}
