extern crate chrono;

use super::schema::{channels, channels_users, users};
use chrono::NaiveDateTime;

#[derive(Debug, Queryable, Associations, Identifiable)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Identifiable, Debug, Queryable, Associations)]
pub struct Channel {
    pub id: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "channels_users"]
#[primary_key(user_id, channel_id)]
pub struct ChannelUser {
    user_id: String,
    channel_id: String,
    pub activity_at: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub username: &'a str,
}

#[derive(Insertable)]
#[table_name = "channels"]
pub struct NewChannel<'a> {
    pub id: &'a str,
}

#[derive(Insertable)]
#[table_name = "channels_users"]
pub struct NewChannelUser<'a> {
    pub user_id: &'a str,
    pub channel_id: &'a str,
}
