// @generated automatically by Diesel CLI.

diesel::table! {
    challenge_types (id) {
        id -> Int2,
        #[max_length = 32]
        name -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    challenges (post_id) {
        post_id -> Int4,
        challenge_type_id -> Int2,
    }
}

diesel::table! {
    comments (id) {
        id -> Int4,
        post_id -> Int4,
        #[max_length = 256]
        body -> Varchar,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Text,
        body -> Text,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    responses (post_id) {
        post_id -> Int4,
        challenge_id -> Int4,
        code -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        profile -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(challenges -> challenge_types (challenge_type_id));
diesel::joinable!(challenges -> posts (post_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(responses -> challenges (challenge_id));
diesel::joinable!(responses -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(
    challenge_types,
    challenges,
    comments,
    posts,
    responses,
    users,
);
