pub mod accounts;
pub mod action_handler;
pub mod address;
pub mod assets;
pub mod config;
pub mod execution_service;
pub mod rollup;
pub mod snapshot;
pub mod storage;
pub mod text;

use astria_eyre::eyre::{Result, WrapErr as _};
use astria_sequencer::BUILD_INFO;
use rollup::Rollup;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    astria_eyre::install().expect("astria eyre hook must be the first hook installed");
    let cfg = config::Config::from_env().unwrap();
    eprintln!(
        "{}",
        serde_json::to_string(&BUILD_INFO)
            .expect("build info is serializable because it contains only unicode fields")
    );
    let mut telemetry_conf = astria_telemetry::configure()
        .set_no_otel(cfg.no_otel)
        .set_force_stdout(cfg.force_stdout)
        .set_pretty_print(cfg.pretty_print)
        .set_filter_directives(&cfg.log);
    if !cfg.no_metrics {
        telemetry_conf = telemetry_conf.set_metrics(&cfg.composer_addr, "sequencer");
    }

    let (_metrics, _telemetry_guard) = match telemetry_conf
        .try_init::<astria_sequencer::Metrics>(&())
        .wrap_err("failed to setup telemetry")
    {
        Err(e) => {
            eprintln!("initializing sequencer failed:\n{e:?}");
            return Ok(());
        }
        Ok(metrics_and_guard) => metrics_and_guard,
    };

    if cfg
        .db_filepath
        .try_exists()
        .context("failed checking for existence of db storage file")?
    {
        info!(
            path = %cfg.db_filepath.display(),
            "opening storage db"
        );
    } else {
        info!(
            path = %cfg.db_filepath.display(),
            "creating storage db"
        );
    }
    Rollup::run_until_stopped(cfg).await?;
    astria_eyre::eyre::Ok(())
}
