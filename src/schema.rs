// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        content -> Nullable<Text>,
        image_url -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        user_type -> Varchar,
        ai_profile -> Nullable<Jsonb>,
        created_at -> Timestamp,
        session_token -> Nullable<Varchar>,
    }
}

diesel::joinable!(messages -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    users,
);
