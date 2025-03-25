use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait PriceOracle: Sync + Send {
    async fn get_price(&self, pool_id: &str) -> anyhow::Result<f64>;
}

#[derive(Clone, Debug)]
pub struct BlufinPriceOracle {
    pub api_url: String,
    pub client: reqwest::Client,
}

impl BlufinPriceOracle {
    pub fn new(api_url: String) -> Self {
        Self { api_url, client: reqwest::Client::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PoolInfo {
    address: String,
    price: String,
}

#[async_trait]
impl PriceOracle for BlufinPriceOracle {
    async fn get_price(&self, pool_id: &str) -> anyhow::Result<f64> {
        tracing::info!("Getting price for pool {}", pool_id);

        let url =
            format!("{}/api/v1/pools/info?pools={}", self.api_url.trim_end_matches('/'), pool_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch pool info: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status code: {}",
                response.status()
            ));
        }

        let pools: Vec<PoolInfo> = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse pool info response: {}", e))?;

        let pool_price = pools
            .iter()
            .find(|pool| pool.address == pool_id)
            .map(|pool| pool.price.parse::<f64>())
            .ok_or_else(|| anyhow::anyhow!("Pool {} not found in API response", pool_id))?
            .map_err(|e| anyhow::anyhow!("Failed to parse price value: {}", e))?;

        Ok(pool_price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_price() {
        let oracle = BlufinPriceOracle::new("https://swap.api.sui-prod.bluefin.io".to_string());
        let price = oracle
            .get_price("0x3b585786b13af1d8ea067ab37101b6513a05d2f90cfe60e8b1d9e1b46a63c4fa")
            .await
            .unwrap();
        println!("Price: {}", price);
        assert!(price > 0.0);
    }
}
