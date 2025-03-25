use std::sync::Arc;

use async_trait::async_trait;
use sui_types::base_types::SuiAddress;

use super::price_oracle::PriceOracle;
use crate::{
    bluefin::models::get_active_positions_by_sender,
    postgres::PgPool,
    services::dex::{DexInterface, RepositionOptions},
    signer::Storage,
};

#[derive(Clone, Debug)]
pub struct ManagedPosition {
    pub position_id: String,
    pub pool_id: String,
    pub user: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[async_trait]
pub trait RepositionManager {
    async fn run(&self);
}

pub struct RunConfig {
    pub poll_interval_ms: u64,
    pub price_change_threshold: f64,
}

pub struct RepositionManagerImpl {
    pub db_pool: Arc<PgPool>,
    pub client: Arc<dyn DexInterface>,
    pub price_oracle: Arc<dyn PriceOracle>,
    pub config: RunConfig,
    pub signer_storage: Arc<dyn Storage>,
}

#[async_trait]
impl RepositionManager for RepositionManagerImpl {
    async fn run(&self) {
        tracing::info!("Starting reposition manager");
        loop {
            let addresses = self.signer_storage.get_all_addresses().unwrap_or_default();
            if addresses.is_empty() {
                tracing::info!("No addresses found, sleeping for 5 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            for address in addresses {
                if let Ok(positions) = self.get_positions(address.clone()).await {
                    for position in positions {
                        let price =
                            self.get_position_price(position.clone()).await.unwrap_or_default();
                        let price_change = price - position.tick_lower as f64;
                        if price_change.abs() > self.config.price_change_threshold {
                            tracing::info!(
                                "Repositioning position {:?} due to price change: {}",
                                position,
                                price_change
                            );
                            let options = RepositionOptions {
                                pool_id: position.pool_id.clone(),
                                position_id: position.position_id.clone(),
                            };
                            let signer =
                                self.signer_storage.get_signer_by_address(&address).unwrap();
                            self.client
                                .reposition(&signer, options)
                                .await
                                .unwrap_or_else(|e| tracing::error!("Failed to reposition: {}", e));
                        }
                    }
                }
            }
            tracing::info!("Sleeping for {} ms", self.config.poll_interval_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(self.config.poll_interval_ms))
                .await;
        }
    }
}

impl RepositionManagerImpl {
    pub fn new(
        config: RunConfig,
        db_pool: Arc<PgPool>,
        client: Arc<dyn DexInterface>,
        price_oracle: Arc<dyn PriceOracle>,
        signer_storage: Arc<dyn Storage>,
    ) -> Self {
        Self { db_pool, client, price_oracle, config, signer_storage }
    }

    pub async fn get_position_price(&self, position: ManagedPosition) -> anyhow::Result<f64> {
        println!("Getting price for position {:?}", position);
        self.get_pool_price(position.pool_id.clone()).await
    }

    pub async fn get_pool_price(&self, pool_id: String) -> anyhow::Result<f64> {
        println!("Getting price for pool {}", pool_id.to_string());
        self.price_oracle.get_price(&pool_id).await
    }

    pub async fn get_positions(&self, address: SuiAddress) -> anyhow::Result<Vec<ManagedPosition>> {
        let positions = get_active_positions_by_sender(&self.db_pool, &address.to_string())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(positions
            .into_iter()
            .map(|p| ManagedPosition {
                position_id: p.position_id.clone(),
                pool_id: p.pool_id.clone(),
                user: p.sender.clone(),
                tick_lower: p.tick_lower,
                tick_upper: p.tick_upper,
            })
            .collect())
    }
}
