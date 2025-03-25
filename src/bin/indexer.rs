use autoliquid_be::bluefin::indexer::Config;
use autoliquid_be::bluefin::run_indexer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        // Filter events based on the RUST_LOG environment variable
        // or fall back to a default level like "info"
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,autoliquid_be=debug,indexer=debug")),
        )
        // Format the output with timestamps and colors
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    let config = Config {
        remote_store_url: "https://checkpoints.mainnet.sui.io".to_string(),
        db_url: "postgresql://postgres:postgres@localhost:5432/autoliquid-db".to_string(),
        checkpoints_path: None,
        sui_rpc_url: "https://fullnode.mainnet.sui.io:443".to_string(),
        package_id: "0x6c796c3ab3421a68158e0df18e4657b2827b1f8fed5ed4b82dba9c935988711b"
            .to_string(), // Bluefin 18
        start_checkpoint: 126529164,
        concurrency: 2,
        metric_port: 9090,
    };

    tracing::info!("Running indexer with config: {:?}", config);

    match run_indexer(config).await {
        Ok(_) => tracing::info!("Indexer completed successfully"),
        Err(e) => tracing::error!("Indexer failed: {}", e),
    }
}
