use std::path::PathBuf;

use config::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
    /// The name of the rollup
    pub rollup_name: String,
    /// The block height from which to read from the sequencer
    pub sequencer_genesis_block_height: u32,
    /// The celestia genesis block height
    pub celestia_genesis_block_height: u32,
    /// The maximun variance in blocks between firm and soft
    pub celestia_block_variance: u64,
    /// The path to penumbra storage db.
    pub db_filepath: PathBuf,
    /// Log level: debug, info, warn, or error
    pub log: String,
    /// tracing otel usage
    pub no_otel: bool,
    /// metrics
    pub no_metrics: bool,
    /// The gRPC endpoint
    pub execution_grpc_addr: String,
    /// Forces writing trace data to stdout no matter if connected to a tty or not.
    pub force_stdout: bool,
    /// Writes a human readable format to stdout instead of JSON formatted OTEL trace data.
    pub pretty_print: bool,
    /// The address of the Composer service.
    pub composer_addr: String,
    /// The endpoint which will be listened on for serving prometheus metrics
    pub metrics_http_listener_addr: String,
}

impl Config {
    /// Load configuration from environment variables and `.env` file.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load environment variables from `.env`
        dotenv::dotenv().ok();

        // Initialize the config loader
        let mut settings = config::Config::builder();

        // Merge environment variables into the configuration
        settings = settings.add_source(config::Environment::default());

        // Build the configuration and deserialize into the `Config` struct
        settings.build()?.try_deserialize::<Self>()
    }
}
