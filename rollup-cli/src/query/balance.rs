use astria_core::primitive::v1::{asset, Address};
use color_eyre::eyre;

#[derive(clap::Args, Debug)]
pub(super) struct Command {
    // The address to get the balance of
    address: Address,
    /// The url of the Rollup node
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
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        let response = reqwest::Client::new()
            .get(format!(
                "{}/get_account_balance/{}/{}",
                self.rollup_url, self.address, self.asset
            ))
            .send()
            .await
            .unwrap();

        match response.text().await {
            Ok(balance) => {
                println!("Balance: {}{}", balance, self.asset);
                Ok(())
            }
            Err(e) => Err(eyre::eyre!(e)),
        }
    }
}
