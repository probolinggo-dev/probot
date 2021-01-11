use futures::future::join_all;
use telegram_bot::*;
pub struct Broadcast<'a> {
    pub api: &'a Api,
    pub message: String,
    pub chat_ids: Vec<i64>,
}

impl<'a> Broadcast<'a> {
    pub fn new(api: &'a Api, message : &str, chat_ids: Vec<i64>) -> Self {
        Broadcast {api, message: message.to_string(),chat_ids}
    }

    pub async fn send(&self) -> Result<(), BroadcastError> {
        let futures_message = self.chat_ids.iter().map(|chat_id| async move{
            let c_id = ChatId::new(*chat_id);
            self.api.spawn(c_id.text(&self.message))
        }).collect::<Vec<_>>(); 

        join_all(
            futures_message
        ).await;
        Ok(())
    }
}
#[derive(Debug)]
pub enum BroadcastError {
    InternalServer
}