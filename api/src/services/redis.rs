use common::{channel::ORDER_CHANNEL, message::api::MessageFromApi};
use r2d2_redis::{r2d2::PooledConnection, redis::{Commands, PubSub}, RedisConnectionManager};

use crate::errors::{ApiError};

pub type RedisResult = Result<String, r2d2_redis::redis::RedisError>;
pub type RedisServiceResult<T> = Result<T, ApiError>;

pub struct RedisService {
    conn: PooledConnection<RedisConnectionManager>
}

impl RedisService {

    pub fn new(conn: PooledConnection<RedisConnectionManager>) -> Self{
        Self { conn }
    }

    pub fn publish_message_to_engine(&mut self, message:MessageFromApi) -> RedisServiceResult<()>{

        let serialized = serde_json::to_string(&message).map_err(|_|{
            println!("Error while serializing message : {:?}", message);
            ApiError::InternalServerError
        })?;

        let _:RedisResult  = self.conn.lpush(ORDER_CHANNEL, serialized);

        Ok(())
    }

}

pub struct PubSubService<'a>{
    pub_sub: PubSub<'a>,
    channel: &'a str,
}

impl<'a> PubSubService<'a> {

    pub fn new(pub_sub: PubSub<'a>, channel:&'a str) -> Self{
        Self { pub_sub, channel }
    }

    pub fn subscribe(&mut self) -> RedisServiceResult<()>{
        self.pub_sub.subscribe(self.channel).map_err(|_|{
            println!("Error while subscribing to channel : {:?}", self.channel);
            ApiError::InternalServerError
        })?;
        Ok(())
    }

    pub fn get_message_from_engine(&mut self) -> RedisServiceResult<String>{

        let message = self.pub_sub.get_message().map_err(|e|{
            println!("Error while receiving message from engine : {:?}", e);
            ApiError::InternalServerError
        })?;

        let payload: RedisResult = message.get_payload();
        
        match payload {
            Ok(val) => {
                Ok(val)
            },
            Err(e) => {
                println!("Error while getting payload from message : {}", e);
                Err(ApiError::InternalServerError)
            }
        }
    }

    pub fn unsubscribe(&mut self) -> RedisServiceResult<()>{

        let res = self.pub_sub.unsubscribe(self.channel).map_err(|e|{
            println!("Error : {} while unsubscribing from channel : {}", e, self.channel);
            ApiError::InternalServerError
        });

        res
    }
}