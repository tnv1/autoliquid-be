use diesel::{
    Identifiable, Insertable, QueryResult, Queryable, QueryableByName, Selectable,
    data_types::PgTimestamp, sql_query, sql_types::Text,
};
use diesel_async::RunQueryDsl;
use sui_indexer_builder::{LIVE_TASK_TARGET_CHECKPOINT, Task};

use crate::{
    postgres::PgPool,
    schema::{position_updates, progress_store, sui_error_transactions},
};

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug)]
#[diesel(table_name = progress_store, primary_key(task_name))]
pub struct ProgressStore {
    pub task_name: String,
    pub checkpoint: i64,
    pub target_checkpoint: i64,
    pub timestamp: Option<PgTimestamp>,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug)]
#[diesel(table_name = sui_error_transactions, primary_key(txn_digest))]
pub struct SuiErrorTransactions {
    pub txn_digest: String,
    pub sender_address: String,
    pub timestamp_ms: i64,
    pub failure_status: String,
    pub package: String,
    pub cmd_idx: Option<i64>,
}

impl From<ProgressStore> for Task {
    fn from(value: ProgressStore) -> Self {
        Self {
            task_name: value.task_name,
            start_checkpoint: value.checkpoint as u64,
            target_checkpoint: value.target_checkpoint as u64,
            timestamp: value.timestamp.expect("timestamp not set").0 as u64,
            is_live_task: value.target_checkpoint == LIVE_TASK_TARGET_CHECKPOINT,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable, QueryableByName, Debug)]
#[diesel(table_name = position_updates, primary_key(event_digest))]
pub struct PositionUpdate {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub digest: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_digest: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub sender: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub checkpoint: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub checkpoint_timestamp_ms: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub package: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub pool_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub position_id: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub tick_lower: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub tick_upper: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub liquidity: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub price: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_close: bool,
}

pub async fn get_active_positions_by_sender(
    pool: &PgPool,
    sender_filter: &str,
) -> QueryResult<Vec<PositionUpdate>> {
    let mut conn = pool.get().await.map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UnableToSendCommand,
            Box::new(e.to_string()),
        )
    })?;

    let sql = r#"
        SELECT pu.*
        FROM position_updates pu
        INNER JOIN (
            SELECT position_id, MAX(checkpoint) AS max_checkpoint
            FROM position_updates
            GROUP BY position_id
        ) latest
        ON pu.position_id = latest.position_id AND pu.checkpoint = latest.max_checkpoint
        WHERE pu.is_close = false
          AND pu.sender = $1
    "#;

    sql_query(sql).bind::<Text, _>(sender_filter).load::<PositionUpdate>(&mut conn).await
}

#[cfg(test)]
mod tests {
    use crate::postgres::get_connection_pool;

    use super::*;

    #[tokio::test]
    async fn test_get_active_positions_by_sender() {
        let pg_pool = get_connection_pool(
            "postgresql://postgres:postgres@localhost:5432/autoliquid-db".into(),
        )
        .await;
        let sender =
            "0x7c90478c4bc8e785e159582a2136a2cdc321fdacae37601ef7615d7552bf6ee3".to_string();
        let actives = get_active_positions_by_sender(&pg_pool, &sender).await.unwrap();
        println!("Number of active positions: {}", actives.len());
        println!("Active positions: {:#?}", actives[0]);
    }
}
