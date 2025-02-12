use astria_core::primitive::v1::{asset, Address};
use color_eyre::eyre::{self, WrapErr as _};
use rollup_core::transaction::v1::{action::Transfer, Action};

use crate::utils::submit_transaction;

#[derive(clap::Args, Debug)]
pub(super) struct Command {
    // The address of the Rollup account to send amount to
    to_address: Address,
    // The amount being sent
    #[arg(long)]
    amount: u128,
    /// The bech32m prefix that will be used for constructing addresses using the private key
    #[arg(long, default_value = "astria")]
    prefix: String,
    /// The private key of account being sent from
    #[arg(long, env = "PRIVATE_KEY")]
    // TODO: https://github.com/astriaorg/astria/issues/594
    // Don't use a plain text private, prefer wrapper like from
    // the secrecy crate with specialized `Debug` and `Drop` implementations
    // that overwrite the key on drop and don't reveal it when printing.
    private_key: String,
    /// The url of the Sequencer node
    #[arg(
        long,
        env = "ROLLUP_URL",
        default_value = crate::DEFAULT_SEQUENCER_RPC
    )]
    rollup_url: String,
    /// The chain id of the rollup chain being used
    #[arg(
        long = "chain-id",
        env = "ROLLUP_CHAIN_ID",
        default_value = crate::DEFAULT_SEQUENCER_CHAIN_ID
    )]
    chain_id: String,
    /// The asset to transer.
    #[arg(long, default_value = "nria")]
    asset: asset::Denom,
    /// The asset to pay the transfer fees with.
    #[arg(long, default_value = "nria")]
    fee_asset: asset::Denom,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        let res = submit_transaction(
            self.rollup_url.as_str(),
            self.chain_id.clone(),
            &self.prefix,
            self.private_key.as_str(),
            Action::Transfer(Transfer {
                to: self.to_address,
                amount: self.amount,
                asset: self.asset.clone(),
                fee_asset: self.fee_asset.clone(),
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
