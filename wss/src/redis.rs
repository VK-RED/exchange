use std::{collections::{HashMap, HashSet}, sync::Arc};

use redis::{aio::MultiplexedConnection, Msg, PushInfo, RedisError};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::AppState;

#[derive(Clone)]
pub struct RedisManager{
    conn: MultiplexedConnection,
    channels_and_users: HashMap<String, HashSet<String>>,
}   

impl RedisManager {

    pub async fn new() -> (Self, UnboundedReceiver<PushInfo>) {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set !");

        // resp3 protocol needed for pubsubs
        let address = format!("{}/?protocol=resp3",redis_url);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);
        let client = redis::Client::open(address).expect("Failed to connect to  Redis !");

        let conn = client.get_multiplexed_async_connection_with_config(&config)
        .await.expect("Failed to get Multiplexed conn");

        let channels_and_users = HashMap::new();

        (Self {
            conn,
            channels_and_users,
        }, rx)
    }

    pub async fn subscribe(&mut self, channel:&str, user_id:String) -> Result<(), RedisError>{
        
        // check if there is a channel exists and subscribe

        let try_users = self.channels_and_users.get_mut(channel);
        
        match try_users {

            Some(users) => {
                println!("{} subscribing to the channel {}", user_id, channel);
                users.insert(user_id);
            },
            None => {
                println!("{} subscribing to the channel {} and new conn opened to redis", user_id, channel);

                let mut users = HashSet::new();
                users.insert(user_id);
                self.channels_and_users.insert(channel.to_string(), users);

                self.conn.subscribe(channel).await?;

            }
        }

        Ok(())
        
    }

    pub async fn unsubcribe(&mut self, channel:&str, user_id:&str) -> Result<(), RedisError>{
        
        let try_users = self.channels_and_users.get_mut(channel);

        if let Some(users) = try_users {
            println!("unsubscribing {} from channel {}", user_id, channel);
            users.remove(user_id);

            // if no one is present on the channel then unsubscribe from the channel
            if users.len() == 0 {
                println!("unsunscribing from redis channel : {}", channel);
                self.conn.unsubscribe(channel).await?;
                self.channels_and_users.remove(channel);
            }
        }

        Ok(())

    }

    pub async fn unsubscribe_channels(&mut self, channels:&HashSet<String>, user_id:&str){

        for channel in channels {
            // TODO: PARALLELIZE IT
            let res = self.unsubcribe(channel, user_id).await;

            if let Err(e) = res{
                println!("error : {} while unsubscribing all channels for user : {}", e, user_id);
            }
        }
    }

    pub async fn broadcast_message_to_users(
        mut rx:UnboundedReceiver<PushInfo>,
        app_state: Arc<Mutex<AppState>>
    ){
        
        while let Some(push_info) = rx.recv().await {

            let try_msg = Msg::from_push_info(push_info);

            if let Some(msg) = try_msg {
                let channel = msg.get_channel_name();

                let try_message = msg.get_payload::<String>();

                match try_message {
                    Ok(message) => {
                        println!("Received message : {} on channel : {}", message, channel);

                        let guard = app_state.lock().await;

                        // get the txs of user to send
                        let try_users = guard.redis.channels_and_users.get(channel);

                        match try_users {
                            Some(users) => {
                                guard.user_manager.emit_messages(message, users).await;
                            },
                            None => {
                                println!("no users on the channel : {} to notify !", channel);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Error : {}, while receiving messages on the channel : {}", e, channel);
                    }
                }

            }
        }
    }



}   