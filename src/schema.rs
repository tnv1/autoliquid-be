// @generated automatically by Diesel CLI.

diesel::table! {
    position_updates (event_digest) {
        digest -> Text,
        event_digest -> Text,
        sender -> Text,
        checkpoint -> Int8,
        checkpoint_timestamp_ms -> Int8,
        package -> Text,
        pool_id -> Text,
        position_id -> Text,
        tick_lower -> Int4,
        tick_upper -> Int4,
        liquidity -> Text,
        price -> Text,
        is_close -> Bool,
    }
}

diesel::table! {
    progress_store (task_name) {
        task_name -> Text,
        checkpoint -> Int8,
        target_checkpoint -> Int8,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sui_error_transactions (id) {
        id -> Int4,
        txn_digest -> Text,
        sender_address -> Text,
        timestamp_ms -> Int8,
        failure_status -> Text,
        package -> Text,
        cmd_idx -> Nullable<Int8>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    position_updates,
    progress_store,
    sui_error_transactions,
);
