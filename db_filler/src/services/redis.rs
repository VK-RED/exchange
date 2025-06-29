use common::channel::DB_CHANNEL;
use redis::{aio::MultiplexedConnection, AsyncTypedCommands};

pub struct RedisService {
    conn: MultiplexedConnection,
}

impl RedisService {
    pub async fn new() -> Self {

        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        let connection = redis::Client::open(redis_url).expect("Failed to connect to redis !!");
        let conn = connection.get_multiplexed_tokio_connection().await.expect("Failed to get multiplexed connections");
        Self {
            conn
        }
    }

    pub async fn get_message_from_engine(&mut self) -> Option<String>{
        let res = self.conn.brpop(DB_CHANNEL, 0.0).await;

        match res {
            Ok(result) => {
                match result {
                    Some(val) => {
                        let message = val[1].to_owned();
                        Some(message)
                    },
                    None => {
                        println!("receiveed none data from engine !");
                        None
                    }
                }
            },
            Err(e) => {
                println!("redis error : {}",e);
                None
            }
        }
    }
}