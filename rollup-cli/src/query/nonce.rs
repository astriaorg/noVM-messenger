use astria_core::primitive::v1::{asset, Address};
use color_eyre::eyre::{self, WrapErr as _};
use rollup_core::transaction::v1::{action::Transfer, Action};

use crate::utils::submit_transaction;

#[derive(clap::Args, Debug)]
pub(super) struct Command {
    // The address to get the balance of
    address: Address,
    /// The url of the Sequencer node
    #[arg(
        long,
        env = "ROLLUP_URL",
        default_value = crate::DEFAULT_SEQUENCER_RPC
    )]
    rollup_url: String,
    /// The chain id of the sequencing chain being used
    #[arg(
        long = "chain-id",
        env = "ROLLUP_CHAIN_ID",
        default_value = crate::DEFAULT_SEQUENCER_CHAIN_ID
    )]
    chain_id: String,
    /// The asset to transer.
    #[arg(long, default_value = "nria")]
    asset: asset::Denom,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        let response = reqwest::Client::new()
            .get(format!(
                "{}/get_account_nonce/{}",
                self.rollup_url, self.address
            ))
            .send()
            .await
            .unwrap();

        match response.text().await {
            Ok(nonce) => {
                println!("Nonce: {}", nonce);
                Ok(())
            }
            Err(e) => Err(eyre::eyre!(e)),
        }
    }
}
