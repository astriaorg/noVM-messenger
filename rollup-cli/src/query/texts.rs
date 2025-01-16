use astria_core::primitive::v1::{asset, Address};
use color_eyre::eyre::{self, Ok, WrapErr as _};
use rollup_core::transaction::v1::{action::Transfer, Action};
use serde_json::{json, Value};
use tracing_subscriber::fmt::format::json;

use crate::utils::submit_transaction;

#[derive(clap::Args, Debug)]
pub(super) struct Command {
    /// The url of the Rollup node
    #[arg(
        long,
        env = "ROLLUP_URL",
        default_value = crate::DEFAULT_SEQUENCER_RPC
    )]
    rollup_url: String,
}

impl Command {
    pub(super) async fn run(self) -> eyre::Result<()> {
        let response = reqwest::Client::new()
            .get(format!("{}/recent", self.rollup_url))
            .send()
            .await
            .unwrap();

        let recents: Value = response.json().await?;

        // Iterate through the array of objects
        if let Some(array) = recents.as_array() {
            for item in array {
                let sender = item
                    .get("sender")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown");
                let message = item.get("message").and_then(|m| m.as_str()).unwrap_or("");
                println!("Sender: {}, Message: {}", sender, message);
            }
        } else {
            eprintln!("Response is not an array");
        }
        Ok(())
    }
}
