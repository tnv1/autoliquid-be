use std::sync::Arc;

use async_trait::async_trait;
use sui_types::base_types::{ObjectID, SuiAddress};

use super::price_oracle::PriceOracle;
use crate::postgres::PgPool;
use crate::services::dex::{DexInterface, RepositionOptions};
use crate::signer::Storage;

#[derive(Clone, Debug)]
pub struct ManagedPosition {
    pub position_id: ObjectID,
    pub pool_id: ObjectID,
    pub user: SuiAddress,
    pub token_a: String,
    pub token_b: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub status: String,
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
        tracing::info!("Service is running");
        loop {
            if let Ok(positions) = self.get_all_active_positions().await {
                for position in positions {
                    if let Ok(price) = self.get_position_price(position.clone()).await {
                        println!("Price for position {:?} is {}", position, price);
                    }
                    let signer = self.signer_storage.get_signer_by_address(&position.user).unwrap();
                    self.client
                        .reposition(
                            &signer,
                            RepositionOptions {
                                pool_id: "".to_string(),
                                position_id: "".to_string(),
                            },
                        )
                        .await
                        .unwrap();
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

    pub async fn get_active_positions(
        &self,
        user: SuiAddress,
    ) -> anyhow::Result<Vec<ManagedPosition>> {
        println!("Getting user positions for {}", user.to_string());
        Ok(vec![])
    }

    pub async fn get_all_active_positions(&self) -> anyhow::Result<Vec<ManagedPosition>> {
        println!("Getting all active positions");
        Ok(vec![])
    }

    pub async fn get_position_price(&self, position: ManagedPosition) -> anyhow::Result<f64> {
        println!("Getting price for position {:?}", position);
        self.get_pool_price(position.pool_id.clone().to_hex()).await
    }

    pub async fn get_pool_price(&self, pool_id: String) -> anyhow::Result<f64> {
        println!("Getting price for pool {}", pool_id.to_string());
        self.price_oracle.get_price(&pool_id).await
    }
}
