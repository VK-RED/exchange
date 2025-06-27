use common::message::{db_filler::{AddOrderToDb, DbFillerMessage, Trade, UpdateOrder}, engine::MessageFromEngine};
use r2d2_redis::{r2d2::{Pool, PooledConnection}, redis::{Commands, RedisError}, RedisConnectionManager};

use crate::errors::EngineError;

pub type RedisResponse = Result<(), r2d2_redis::redis::RedisError>;

// polls the message from this channel, published by the API
const ORDER_CHANNEL: &'static str = "orders";
const DB_CHANNEL:&'static str = "db_filler";

#[derive(Debug)]
pub struct RedisService {
    pool: Pool<RedisConnectionManager>,
}

impl RedisService {
    
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_conn(&self) -> Option<PooledConnection<RedisConnectionManager>>{
        let conn_res = self.pool.get();

        if let Err(e) = conn_res {
            println!("Error : {e} while acquiring redis connection from pool in update_db orders");
            return None;
        }

        let conn = conn_res.unwrap();
        Some(conn)
    }

    pub fn publish_to_db_filler(&self, mut conn: PooledConnection<RedisConnectionManager>, message:DbFillerMessage){

        let serialized_message = serde_json::to_string(&message);

        match serialized_message{
            Ok(serialized) => {

                let redis_res:RedisResponse = conn.lpush(DB_CHANNEL, serialized);

                if let Err(e) = redis_res {
                    println!("Error:{} while publishing to db channel from update_db_orders", e);
                }
            },
            Err(e) => {
                println!("error while serializing message for db filler in update db orders : {} ", e);
            }
        }
    }

    pub fn update_db_orders(
        &self,
        add_order: Option<AddOrderToDb>,
        update_orders: Vec<UpdateOrder>
    ){
        let conn_res = self.get_conn();

        if let None = conn_res {
            return;
        }

        if let None = add_order {
            println!("some error occurred add_order is None while publishing db orders");
            println!("publishing the updated_orders to db_filler  : {:?}", update_orders)
        }

        let conn = conn_res.unwrap();
        let message = DbFillerMessage::AddAndUpdateOrders { add_order, update_orders };
        self.publish_to_db_filler(conn, message);

    }

    
    pub fn publish_trades_to_db(&self, trades: Vec<Trade>){
        let conn_res = self.get_conn();

        if let None = conn_res {
            return;
        }

        let conn = conn_res.unwrap();
        let message = DbFillerMessage::AddTrade(trades);
        self.publish_to_db_filler(conn, message);
        
    }

    pub fn publish_cancel_order_updates(&self, orders:Vec<String>){
        let conn_res = self.get_conn();

        if let None = conn_res {
            return;
        }

        let conn = conn_res.unwrap();
        let message = DbFillerMessage::UpdateCancelOrders(orders);
        self.publish_to_db_filler(conn, message);
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
