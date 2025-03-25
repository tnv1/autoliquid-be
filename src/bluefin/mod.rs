use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use indexer::{BluefinDataMapper, BluefinStorage, Config};
use metrics::IndexerMetrics;
use mysten_metrics::start_prometheus_server;
use sui_data_ingestion_core::DataIngestionMetrics;
use sui_indexer_builder::indexer_builder::IndexerBuilder;
use sui_indexer_builder::progress::{OutOfOrderSaveAfterDurationPolicy, ProgressSavingPolicy};
use sui_indexer_builder::sui_datasource::SuiCheckpointDatasource;
use sui_sdk::SuiClientBuilder;
use sui_types::base_types::ObjectID;

use crate::postgres::get_connection_pool;

pub mod events;
pub mod indexer;
pub mod metrics;
pub mod models;

pub async fn run_indexer(config: Config) -> anyhow::Result<()> {
    // Init metrics server
    let metrics_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.metric_port);
    let registry_service = start_prometheus_server(metrics_address);
    let registry = registry_service.default_registry();
    mysten_metrics::init_metrics(&registry);
    tracing::info!("Metrics server started at port {}", config.metric_port);

    let indexer_meterics = IndexerMetrics::new(&registry);
    let ingestion_metrics = DataIngestionMetrics::new(&registry);

    let db_url = config.db_url.clone();
    let pg_pool = get_connection_pool(db_url).await;
    let policy = ProgressSavingPolicy::OutOfOrderSaveAfterDuration(
        OutOfOrderSaveAfterDurationPolicy::new(tokio::time::Duration::from_secs(30)),
    );
    let datastore = BluefinStorage::new(pg_pool, policy);
    let sui_client = Arc::new(SuiClientBuilder::default().build(config.sui_rpc_url.clone()).await?);
    let sui_checkpoint_datasource = SuiCheckpointDatasource::new(
        config.remote_store_url,
        sui_client,
        config.concurrency as usize,
        config.checkpoints_path.map(|p| p.into()).unwrap_or(tempfile::tempdir()?.into_path()),
        config.start_checkpoint,
        ingestion_metrics.clone(),
        Box::new(indexer_meterics.clone()),
    );

    let indexer = IndexerBuilder::new(
        "BluefinIndexer",
        sui_checkpoint_datasource,
        BluefinDataMapper {
            metrics: indexer_meterics.clone(),
            package_id: ObjectID::from_hex_literal(&config.bluefin_spot_package_id.clone())
                .unwrap_or_else(|err| panic!("Failed to parse bluefin package ID: {}", err)),
        },
        datastore,
    )
    .build();

    tracing::info!("Starting indexer");
    indexer.start().await?;
    tracing::info!("Stopped indexer");
    Ok(())
}
