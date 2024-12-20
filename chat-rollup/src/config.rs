use std::path::PathBuf;

use config::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
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
