table! {
    statistics (id) {
        id -> Int4,
        timestamp -> Timestamp,
        count -> Int4,
        user_id -> Int4,
    }
}

table! {
    tokens (id) {
        id -> Int4,
        token -> Varchar,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        frozen -> Bool,
        visible -> Bool,
    }
}

joinable!(statistics -> users (user_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(statistics, tokens, users,);
