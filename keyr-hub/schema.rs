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
    }
}

joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    tokens,
    users,
);
