use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::digests::TransactionDigest;

#[derive(Clone, Debug)]
pub enum ProcessedTxnData {
    Position(PositionUpdate),
    Error(SuiTxnError),
}

#[derive(Clone, Debug)]
pub struct PositionUpdate {
    pub digest: String,
    pub event_digest: String,
    pub sender: String,
    pub checkpoint: u64,
    pub checkpoint_timestamp_ms: u64,
    pub package: String,
    pub pool_id: ObjectID,
    pub position_id: ObjectID,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub price: f64,
    pub is_close: bool,
}

#[derive(Clone, Debug)]
pub struct SuiTxnError {
    pub tx_digest: TransactionDigest,
    pub sender: SuiAddress,
    pub timestamp_ms: u64,
    pub failure_status: String,
    pub package: String,
    pub cmd_idx: Option<u64>,
}
