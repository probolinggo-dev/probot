extern crate dotenv;

use dotenv::dotenv;
use std::env;

use futures::StreamExt;
use probot::*;
use telegram_bot::*;
use bc::*;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();
    let connection = establish_connection();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = &Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("{:?}", data);
                let cloned_message = message.clone();
                let user_id = &cloned_message.from.id.to_string();
                let channel_id = &cloned_message.chat.id().to_string();
                let username = &cloned_message.from.username.unwrap();
                let cache_key = channel_id.to_owned() + user_id;

                // record channel activity
                if user_id != channel_id {
                    if !get_activity_cache(&cache_key) {
                        record_activity(&connection, channel_id, user_id, username).unwrap();
                        set_activity_cache(&cache_key).unwrap();
                    }

                    // respond to @here
                    if data.contains("@here ")
                        || data == "@here"
                        || data.contains(" @here")
                        || data.contains("\n@here")
                        || data.contains("@here\n")
                    {
                        let users = get_channel_users(&connection, channel_id);
                        let usernames: Vec<String> = users
                            .into_iter()
                            .map(|user| "@".to_owned() + &user.username)
                            .rev()
                            .collect();
                        let usernames = usernames.join(" ");

                        api.send(message.text_reply(format!("{}", usernames)))
                            .await?;
                    } else if data.contains("/bc") || data.contains("@bc") {
                        let users = get_channel_users(&connection, channel_id);
                        let users: Vec<i64> = users.iter().flat_map(|u| u.id.parse()).collect();
                        let broadcast = Broadcast::new(api, &data[3..], users);
                        broadcast.send().await?;
                    }
                }
            }
        }
    }
    Ok(())
}
#[derive(Debug)]
pub enum BotError {
    Telegram(telegram_bot::Error),
    Broadcast(BroadcastError)
}

impl From<telegram_bot::Error> for BotError {
    fn from(error: telegram_bot::Error) -> Self {
        BotError::Telegram(error)
    }
}

impl From<BroadcastError> for BotError {
    fn from(error: BroadcastError) -> Self {
        BotError::Broadcast(error)
    }
}