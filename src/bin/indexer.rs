use autoliquid_be::blufin::config::IndexerConfig;
use autoliquid_be::blufin::run_indexer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = IndexerConfig {
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

    run_indexer(config).await.unwrap();
}
