#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate redis;

pub mod models;
pub mod schema;

use self::models::{Channel, ChannelUser, NewChannel, NewChannelUser, NewUser, User};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use redis::Commands;
use std::env;
use std::error::Error;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn get_user<'a>(conn: &PgConnection, id: &'a str) -> Option<User> {
    use self::schema::users::dsl::users;
    users.find(id).first(conn).optional().unwrap()
}

pub fn set_activity_cache<'a>(key: &'a str) -> Result<(), Box<dyn Error>> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;

    con.set_ex(key, 1u8, 3600)?;

    Ok(())
}

pub fn get_activity_cache<'a>(key: &'a str) -> bool {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).unwrap();
    let mut con = client.get_connection().unwrap();

    let result: u8 = con.get(key).unwrap_or(0u8);

    result == 1
}

fn create_user<'a>(
    conn: &PgConnection,
    id: &'a str,
    username: &'a str,
) -> Result<(), Box<dyn Error>> {
    use schema::users;
    let new_user = NewUser { id, username };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
        .expect("error creating new user");

    Ok(())
}

fn get_channel<'a>(conn: &PgConnection, id: &'a str) -> Option<Channel> {
    use self::schema::channels::dsl::channels;

    channels.find(id).first(conn).optional().unwrap()
}

fn create_channel<'a>(conn: &PgConnection, id: &'a str) -> Result<(), Box<dyn Error>> {
    use schema::channels;

    let new_channel = NewChannel { id };

    diesel::insert_into(channels::table)
        .values(&new_channel)
        .execute(conn)
        .expect("error creating new channel");
    Ok(())
}

fn get_channel_user<'a>(
    conn: &PgConnection,
    user_id: &'a str,
    channel_id: &'a str,
) -> Option<ChannelUser> {
    use self::schema::channels_users::dsl::channels_users;

    channels_users
        .find((user_id, channel_id))
        .first(conn)
        .optional()
        .unwrap()
}

fn update_relation<'a>(
    conn: &PgConnection,
    user_id: &'a str,
    channel_id: &'a str,
) -> Result<(), Box<dyn Error>> {
    use self::schema::channels_users::dsl::channels_users;
    let now = chrono::offset::Local::now().naive_local();

    diesel::update(
        channels_users.filter(
            schema::channels_users::user_id
                .eq(user_id)
                .and(schema::channels_users::channel_id.eq(channel_id)),
        ),
    )
    .set(schema::channels_users::activity_at.eq(now))
    .execute(conn)
    .unwrap();

    Ok(())
}

fn create_channel_user<'a>(
    conn: &PgConnection,
    user_id: &'a str,
    channel_id: &'a str,
) -> Result<(), Box<dyn Error>> {
    use schema::channels_users;

    let new_channel_user = NewChannelUser {
        user_id,
        channel_id,
    };

    diesel::insert_into(channels_users::table)
        .values(&new_channel_user)
        .execute(conn)
        .expect("error creating new channel");
    Ok(())
}

pub fn get_channel_users<'a>(conn: &PgConnection, channel_id: &'a str) -> Vec<User> {
    use diesel::pg::expression::dsl::any;
    use schema::channels_users;
    use schema::users;

    let channel = get_channel(conn, channel_id).unwrap();

    let channel_user_ids = ChannelUser::belonging_to(&channel).select(channels_users::user_id);
    users::table
        .filter(users::id.eq(any(channel_user_ids)))
        .load::<User>(conn)
        .expect("error get list of users")
}

pub fn record_activity<'a>(
    conn: &PgConnection,
    channel_id: &'a str,
    user_id: &'a str,
    username: &'a str,
) -> Result<(), Box<dyn Error>> {
    if let None = get_channel(conn, channel_id) {
        create_channel(conn, channel_id).unwrap();
    }

    if let None = get_user(conn, user_id) {
        create_user(conn, user_id, username).unwrap();
    }

    if let Some(_channel_user) = get_channel_user(conn, user_id, channel_id) {
        update_relation(conn, user_id, channel_id).unwrap();
    } else {
        create_channel_user(conn, user_id, channel_id).unwrap();
    }

    Ok(())
}
