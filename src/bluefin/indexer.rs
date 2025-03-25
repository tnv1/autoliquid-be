use anyhow::Error;
use async_trait::async_trait;
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper, TextExpressionMethods,
    dsl::now,
};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::{Deserialize, Serialize};
use sui_indexer_builder::{
    LIVE_TASK_TARGET_CHECKPOINT, Task, Tasks,
    indexer_builder::{DataMapper, IndexerProgressStore, Persistent},
    progress::ProgressSavingPolicy,
    sui_datasource::CheckpointTxnData,
};
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    digests::TransactionDigest,
    effects::TransactionEffectsAPI,
    event::Event,
    execution_status::ExecutionStatus,
    full_checkpoint_content::CheckpointTransaction,
    transaction::{Command, TransactionDataAPI},
};

use super::{metrics::IndexerMetrics, models};
use crate::{
    bluefin::{
        events::{PositionClosed, PositionOpened},
        models::SuiErrorTransactions,
    },
    postgres::PgPool,
    schema::{
        self,
        progress_store::{columns, dsl},
        sui_error_transactions,
    },
};

pub const POSITION_OPENED_EVENT: &str = "PositionOpened";
pub const POSITION_CLOSED_EVENT: &str = "PositionClosed";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub remote_store_url: String,
    pub db_url: String,
    pub checkpoints_path: Option<String>,
    pub sui_rpc_url: String,
    pub bluefin_spot_package_id: String,
    pub start_checkpoint: u64,
    pub concurrency: u64,
    pub metric_port: u16,
}

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

#[derive(Clone)]
pub struct BluefinStorage {
    pub pool: PgPool,
    save_progress_policy: ProgressSavingPolicy,
}

impl BluefinStorage {
    pub fn new(pool: PgPool, save_progress_policy: ProgressSavingPolicy) -> Self {
        Self { pool, save_progress_policy }
    }

    pub async fn get_largest_backfill_task_target_checkpoint(
        &self,
        prefix: &str,
    ) -> anyhow::Result<Option<u64>> {
        let mut conn = self.pool.get().await?;
        let cp = dsl::progress_store
            .select(columns::target_checkpoint)
            .filter(columns::task_name.like(format!("{prefix} - %")))
            .filter(columns::target_checkpoint.ne(i64::MAX))
            .order_by(columns::target_checkpoint.desc())
            .first::<i64>(&mut conn)
            .await
            .optional()?;
        Ok(cp.map(|c| c as u64))
    }
}

#[async_trait]
impl Persistent<ProcessedTxnData> for BluefinStorage {
    async fn write(&self, data: Vec<ProcessedTxnData>) -> Result<(), Error> {
        if data.is_empty() {
            tracing::info!("No data to write.");
            return Ok(());
        }

        use futures::future;

        let mut error_transactions_batch = vec![];
        let mut positions_batch = vec![];

        for d in data {
            match d {
                ProcessedTxnData::Error(e) => error_transactions_batch.push(SuiErrorTransactions {
                    txn_digest: e.tx_digest.to_string(),
                    sender_address: e.sender.to_string(),
                    timestamp_ms: e.timestamp_ms as i64,
                    failure_status: e.failure_status.to_string(),
                    package: e.package.to_string(),
                    cmd_idx: e.cmd_idx.map(|idx| idx as i64),
                }),
                ProcessedTxnData::Position(position_update) => {
                    positions_batch.push(models::PositionUpdate {
                        digest: position_update.digest,
                        event_digest: position_update.event_digest,
                        sender: position_update.sender,
                        checkpoint: position_update.checkpoint as i64,
                        checkpoint_timestamp_ms: position_update.checkpoint_timestamp_ms as i64,
                        package: position_update.package,
                        pool_id: position_update.pool_id.to_string(),
                        position_id: position_update.position_id.to_string(),
                        tick_lower: position_update.tick_lower,
                        tick_upper: position_update.tick_upper,
                        liquidity: position_update.liquidity.to_string(),
                        is_close: position_update.is_close,
                        price: "0".to_string(),
                    });
                }
            }
        }

        let connection = &mut self.pool.get().await?;
        connection
            .transaction(|conn| {
                async move {
                    // Create async tasks for each batch insert
                    let mut tasks = Vec::new();

                    if !error_transactions_batch.is_empty() {
                        tasks.push(
                            diesel::insert_into(sui_error_transactions::table)
                                .values(&error_transactions_batch)
                                .on_conflict_do_nothing()
                                .execute(conn),
                        );
                    }

                    if !positions_batch.is_empty() {
                        tasks.push(
                            diesel::insert_into(schema::position_updates::table)
                                .values(&positions_batch)
                                .on_conflict_do_nothing()
                                .execute(conn),
                        );
                    }

                    // Execute all tasks concurrently
                    let _: Vec<_> = future::try_join_all(tasks).await?;

                    Ok(())
                }
                .scope_boxed()
            })
            .await
    }
}

#[async_trait]
impl IndexerProgressStore for BluefinStorage {
    async fn load_progress(&self, task_name: String) -> anyhow::Result<u64> {
        let mut conn = self.pool.get().await?;
        let cp: Option<models::ProgressStore> = dsl::progress_store
            .find(&task_name)
            .select(models::ProgressStore::as_select())
            .first(&mut conn)
            .await
            .optional()?;
        Ok(cp.ok_or(anyhow::anyhow!("Cannot found progress for task {task_name}"))?.checkpoint
            as u64)
    }

    async fn save_progress(
        &mut self,
        task: &Task,
        checkpoint_numbers: &[u64],
    ) -> anyhow::Result<Option<u64>> {
        if checkpoint_numbers.is_empty() {
            return Ok(None);
        }
        let task_name = task.task_name.clone();
        if let Some(checkpoint_to_save) =
            self.save_progress_policy.cache_progress(task, checkpoint_numbers)
        {
            let mut conn = self.pool.get().await?;
            diesel::insert_into(schema::progress_store::table)
                .values(&models::ProgressStore {
                    task_name,
                    checkpoint: checkpoint_to_save as i64,
                    // Target checkpoint and timestamp will only be written for new entries
                    target_checkpoint: i64::MAX,
                    // Timestamp is defaulted to current time in DB if None
                    timestamp: None,
                })
                .on_conflict(dsl::task_name)
                .do_update()
                .set((
                    columns::checkpoint.eq(checkpoint_to_save as i64),
                    columns::timestamp.eq(now),
                ))
                .execute(&mut conn)
                .await?;
            return Ok(Some(checkpoint_to_save));
        }
        Ok(None)
    }

    async fn get_ongoing_tasks(&self, prefix: &str) -> Result<Tasks, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        // get all unfinished tasks
        let cp: Vec<models::ProgressStore> = dsl::progress_store
            .filter(columns::task_name.like(format!("{prefix} - %")))
            .filter(columns::checkpoint.lt(columns::target_checkpoint))
            .order_by(columns::target_checkpoint.desc())
            .load(&mut conn)
            .await?;
        let tasks = cp.into_iter().map(|d| d.into()).collect();
        Ok(Tasks::new(tasks)?)
    }

    async fn get_largest_indexed_checkpoint(&self, prefix: &str) -> Result<Option<u64>, Error> {
        let mut conn = self.pool.get().await?;
        let cp = dsl::progress_store
            .select(columns::checkpoint)
            .filter(columns::task_name.like(format!("{prefix} - %")))
            .filter(columns::target_checkpoint.eq(i64::MAX))
            .first::<i64>(&mut conn)
            .await
            .optional()?;

        if let Some(cp) = cp {
            Ok(Some(cp as u64))
        } else {
            // Use the largest backfill target checkpoint as a fallback
            self.get_largest_backfill_task_target_checkpoint(prefix).await
        }
    }

    async fn register_task(
        &mut self,
        task_name: String,
        checkpoint: u64,
        target_checkpoint: u64,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        diesel::insert_into(schema::progress_store::table)
            .values(models::ProgressStore {
                task_name,
                checkpoint: checkpoint as i64,
                target_checkpoint: target_checkpoint as i64,
                // Timestamp is defaulted to current time in DB if None
                timestamp: None,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn register_live_task(
        &mut self,
        task_name: String,
        start_checkpoint: u64,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        diesel::insert_into(schema::progress_store::table)
            .values(models::ProgressStore {
                task_name,
                checkpoint: start_checkpoint as i64,
                target_checkpoint: LIVE_TASK_TARGET_CHECKPOINT,
                // Timestamp is defaulted to current time in DB if None
                timestamp: None,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn update_task(&mut self, task: Task) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        diesel::update(dsl::progress_store.filter(columns::task_name.eq(task.task_name)))
            .set((
                columns::checkpoint.eq(task.start_checkpoint as i64),
                columns::target_checkpoint.eq(task.target_checkpoint as i64),
                columns::timestamp.eq(now),
            ))
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct BluefinDataMapper {
    pub metrics: IndexerMetrics,
    pub package_id: ObjectID,
}

impl DataMapper<CheckpointTxnData, ProcessedTxnData> for BluefinDataMapper {
    fn map(
        &self,
        (data, checkpoint_num, timestamp_ms): CheckpointTxnData,
    ) -> Result<Vec<ProcessedTxnData>, anyhow::Error> {
        if !data.input_objects.iter().any(|obj| {
            obj.data.type_().map(|t| t.address() == self.package_id.into()).unwrap_or_default()
        }) {
            return Ok(vec![]);
        }

        self.metrics.total_transactions.inc();

        match &data.events {
            Some(events) => {
                let processed_sui_events =
                    events.data.iter().enumerate().try_fold(vec![], |mut result, (i, ev)| {
                        if let Some(data) = process_sui_event(
                            ev,
                            i,
                            &data,
                            checkpoint_num,
                            timestamp_ms,
                            self.package_id,
                        )? {
                            result.push(data);
                        }
                        Ok::<_, anyhow::Error>(result)
                    })?;
                if !processed_sui_events.is_empty() {
                    tracing::info!(
                        "SUI: Extracted {} bluefin data entries for tx {}.",
                        processed_sui_events.len(),
                        data.transaction.digest()
                    );
                }
                return Ok(processed_sui_events);
            }
            None => {
                if let ExecutionStatus::Failure { error, command } = data.effects.status() {
                    let txn_kind = data.transaction.transaction_data().clone().into_kind();
                    let first_command = txn_kind.iter_commands().next();
                    let package = if let Some(Command::MoveCall(move_call)) = first_command {
                        move_call.package.to_string()
                    } else {
                        "".to_string()
                    };
                    return Ok(vec![ProcessedTxnData::Error(SuiTxnError {
                        tx_digest: *data.transaction.digest(),
                        sender: data.transaction.sender_address(),
                        timestamp_ms,
                        failure_status: error.to_string(),
                        package,
                        cmd_idx: command.map(|idx| idx as u64),
                    })]);
                } else {
                    return Ok(vec![]);
                }
            }
        }
    }
}

pub fn process_sui_event(
    ev: &Event,
    event_index: usize,
    tx: &CheckpointTransaction,
    checkpoint: u64,
    checkpoint_timestamp_ms: u64,
    package_id: ObjectID,
) -> anyhow::Result<Option<ProcessedTxnData>> {
    Ok(if ev.type_.address.to_hex() == *package_id.to_hex() {
        match ev.type_.name.as_str() {
            POSITION_OPENED_EVENT => {
                tracing::info!("Handle PositionOpened event: {:?}", ev);
                let move_event: PositionOpened = bcs::from_bytes(&ev.contents)?;
                let txn_kind = tx.transaction.transaction_data().clone().into_kind();
                let first_command = txn_kind.iter_commands().next();
                let package = if let Some(Command::MoveCall(move_call)) = first_command {
                    move_call.package.to_string()
                } else {
                    "".to_string()
                };
                let mut event_digest = tx.transaction.digest().to_string();
                event_digest.push_str(&event_index.to_string());

                let txn_data = Some(ProcessedTxnData::Position(PositionUpdate {
                    digest: tx.transaction.digest().to_string(),
                    event_digest,
                    sender: tx.transaction.sender_address().to_string(),
                    checkpoint,
                    checkpoint_timestamp_ms,
                    package,
                    pool_id: move_event.pool_id,
                    position_id: move_event.position_id,
                    tick_lower: move_event.tick_lower,
                    tick_upper: move_event.tick_upper,
                    liquidity: 0,
                    is_close: false,
                    price: 0.0,
                }));
                txn_data
            }

            POSITION_CLOSED_EVENT => {
                tracing::info!("Handle PositionClosed event: {:?}", ev);
                let move_event: PositionClosed = bcs::from_bytes(&ev.contents)?;
                let txn_kind = tx.transaction.transaction_data().clone().into_kind();
                let first_command = txn_kind.iter_commands().next();
                let package = if let Some(Command::MoveCall(move_call)) = first_command {
                    move_call.package.to_string()
                } else {
                    "".to_string()
                };
                let mut event_digest = tx.transaction.digest().to_string();
                event_digest.push_str(&event_index.to_string());

                let txn_data = Some(ProcessedTxnData::Position(PositionUpdate {
                    digest: tx.transaction.digest().to_string(),
                    event_digest,
                    sender: tx.transaction.sender_address().to_string(),
                    checkpoint,
                    checkpoint_timestamp_ms,
                    package,
                    pool_id: move_event.pool_id,
                    position_id: move_event.position_id,
                    tick_lower: move_event.tick_lower,
                    tick_upper: move_event.tick_upper,
                    liquidity: 0,
                    is_close: true,
                    price: 0.0,
                }));
                txn_data
            }
            _ => {
                tracing::info!("Not supported events: {:?}", ev);
                None
            }
        }
    } else {
        None
    })
}
