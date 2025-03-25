DROP TABLE IF EXISTS sui_error_transactions;
DROP TABLE IF EXISTS progress_store;

DROP INDEX IF EXISTS idx_position_updates_pool_open;
DROP INDEX IF EXISTS idx_position_updates_sender_open;
DROP INDEX IF EXISTS idx_position_updates_open;
DROP INDEX IF EXISTS idx_position_updates_pool_id;
DROP INDEX IF EXISTS idx_position_updates_sender;

DROP TABLE IF EXISTS position_updates;