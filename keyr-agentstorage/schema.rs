table! {
    staging_area (timestamp) {
        timestamp -> Timestamp,
        count -> Integer,
    }
}

table! {
    summary (since) {
        since -> Timestamp,
        count -> BigInt,
    }
}

allow_tables_to_appear_in_same_query!(
    staging_area,
    summary,
);
