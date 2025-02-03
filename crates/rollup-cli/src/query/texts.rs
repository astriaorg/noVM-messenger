use color_eyre::eyre::{self, Ok};
use serde_json::Value;

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
