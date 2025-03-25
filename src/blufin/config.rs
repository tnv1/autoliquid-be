use serde::{Deserialize, Serialize};

/// config as loaded from `config.yaml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexerConfig {
    pub remote_store_url: String,
    #[serde(default = "default_db_url")]
    pub db_url: String,
    pub checkpoints_path: Option<String>,
    pub sui_rpc_url: String,
    pub package_id: String,
    pub start_checkpoint: u64,
    pub concurrency: u64,
    pub metric_port: u16,
}

pub fn default_db_url() -> String {
    std::env::var("DB_URL").expect("db_url must be set in config or via the $DB_URL env var")
}
