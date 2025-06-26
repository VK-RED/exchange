use common::message::engine::{MessageFromEngine};
use r2d2_redis::{r2d2::Pool, redis::{Commands, RedisError}, RedisConnectionManager};

use crate::errors::EngineError;

pub type RedisResponse = Result<(), r2d2_redis::redis::RedisError>;

// polls the message from this channel, published by the API
const ORDER_CHANNEL: &'static str = "orders";

#[derive(Debug)]
pub struct RedisService {
    pool: Pool<RedisConnectionManager>,
}

impl RedisService {
    
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn publish_message_to_api(
        &self,
        channel:String, 
        message_res:Result<MessageFromEngine, EngineError>){

        let mut conn = self.pool.get().unwrap();

        let serialized = match message_res {
            Err(e) => {
                e.to_error_response().serialize_as_err()
            },
            Ok(message) => {
                message.serialize_data_as_ok()
            }
        };

        let res:RedisResponse = conn.publish(channel, serialized);        

        if let Err(e) = res {
            println!("Error while publishing message to api : {}", e);
        }

    }

    pub fn get_message_from_api(&self) -> Option<Option<String>>{
        let mut conn = self.pool.get().unwrap(); 

        let res: Result<Option<String>,RedisError> = conn.rpop(ORDER_CHANNEL);
    
        match res {
            Ok(message) => {
                Some(message)
            },
            Err(e) => {
                println!("Error while polling from the queue : {}", &e);
                None
            }
        }
    }

}
