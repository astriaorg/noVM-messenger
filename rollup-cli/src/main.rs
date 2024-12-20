use color_eyre::eyre;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_writer(std::io::stderr)
        .init();

    rollup_cli::Cli::run().await
}
