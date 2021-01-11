table! {
    channels (id) {
        id -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

table! {
    channels_users (user_id, channel_id) {
        user_id -> Varchar,
        channel_id -> Varchar,
        activity_at -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

joinable!(channels_users -> channels (channel_id));
joinable!(channels_users -> users (user_id));

allow_tables_to_appear_in_same_query!(
    channels,
    channels_users,
    users,
);
