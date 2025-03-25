use async_trait::async_trait;

#[async_trait]
pub trait PriceOracle: Sync + Send {
    async fn get_price(&self, pool_id: &str) -> anyhow::Result<f64>;
}
