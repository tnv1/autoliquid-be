CREATE TABLE IF NOT EXISTS progress_store (
    task_name TEXT PRIMARY KEY,
    checkpoint BIGINT NOT NULL,
    target_checkpoint BIGINT DEFAULT 9223372036854775807 NOT NULL,
    timestamp TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sui_error_transactions (
    id SERIAL PRIMARY KEY,
    txn_digest TEXT NOT NULL,
    sender_address TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    failure_status TEXT NOT NULL,
    package TEXT NOT NULL,
    cmd_idx BIGINT
);

CREATE TABLE IF NOT EXISTS position_updates (
    digest TEXT NOT NULL,
    event_digest TEXT PRIMARY KEY,
    sender TEXT NOT NULL,
    checkpoint BIGINT NOT NULL,
    checkpoint_timestamp_ms BIGINT NOT NULL,
    package TEXT NOT NULL,
    pool_id TEXT NOT NULL,
    position_id TEXT NOT NULL,
    tick_lower INTEGER NOT NULL,
    tick_upper INTEGER NOT NULL,
    liquidity TEXT NOT NULL,
    price TEXT NOT NULL,
    is_close BOOLEAN NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_position_updates_sender ON position_updates(sender);
CREATE INDEX IF NOT EXISTS idx_position_updates_pool_id ON position_updates(pool_id);
CREATE INDEX IF NOT EXISTS idx_position_updates_open ON position_updates(is_close) WHERE is_close = false;
CREATE INDEX IF NOT EXISTS idx_position_updates_sender_open ON position_updates(sender) WHERE is_close = false;
CREATE INDEX IF NOT EXISTS idx_position_updates_pool_open ON position_updates(pool_id) WHERE is_close = false;
