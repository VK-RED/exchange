use common::{channel::{DB_CHANNEL, ORDER_CHANNEL, USER_CHANNEL}, message::{db_filler::{AddOrderToDb, DbFillerMessage, Trade, UpdateOrder}, engine::{MessageFromEngine, UserMessageFromEngine}, ws::{DepthUpdate, TradeUpdate, WsMessage}}};
use r2d2_redis::{r2d2::{Pool, PooledConnection}, redis::{Commands, RedisError}, RedisConnectionManager};
use rust_decimal::Decimal;

use crate::{errors::EngineError, orderbook::PriceWithDepth};

pub type RedisResponse = Result<(), r2d2_redis::redis::RedisError>;

#[derive(Debug, Clone)]
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

    fn publish_to_db_filler(&self, mut conn: PooledConnection<RedisConnectionManager>, message:DbFillerMessage){

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

    fn publish_to_ws(
        &self, 
        channel:&str,
        mut conn: PooledConnection<RedisConnectionManager>, 
        message:WsMessage
    ){
        let serialized_message = serde_json::to_string(&message);

        match serialized_message{
            Ok(serialized) => {

                let redis_res:RedisResponse = conn.publish(channel, serialized);

                if let Err(e) = redis_res {
                    println!("Error:{} while publishing to wss", e);
                }
            },
            Err(e) => {
                println!("error while serializing message for wss : {} ", e);
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
        channel:&str, 
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

    pub fn get_user_message_from_api(&self) -> Option<String>{

        let mut conn = self.pool.get().unwrap(); 

        let res: Result<Option<String>,RedisError> = conn.rpop(USER_CHANNEL);
    
        match res {
            Ok(message) => {
                message
            },
            Err(e) => {
                println!("Error while polling from the queue : {}", &e);
                None
            }
        }
    }

    pub fn publish_user_message_to_api(
        &self,
        channel:String,
        message_res:Result<UserMessageFromEngine, EngineError>,
    ) {

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

    pub fn publish_ws_trade(&self, market:&str, trades:&Vec<Trade>){

        let channel = format!("trade@{}", market);

        let conn_res = self.get_conn();

        if let None = conn_res {
            return;
        }

        let conn = conn_res.unwrap();

        let trade_updates: Vec<TradeUpdate> = trades.iter().map(|trade| TradeUpdate {
            e: "trade".to_string(),
            p: trade.price,
            q: trade.quantity,
            s: trade.market.clone(),
            t: trade.id,
        }).collect();

        let message = WsMessage::Trade(trade_updates);

        self.publish_to_ws(&channel, conn, message);
        
    }

    pub fn publish_ws_depth(
        &self, 
        market:&str,
        price_w_depth: Option<PriceWithDepth>
    ){

        let channel = format!("depth@{}", market);

        let conn_res = self.get_conn();

        if let None = conn_res {
            return;
        }

        let conn = conn_res.unwrap();

        let depth_update = match price_w_depth {
            Some(depth) => {

                let bids:Vec<[Decimal;2]> = depth.updated_bids.iter()
                .map(|(price, qty)| [*price, *qty])
                .collect();
                
                let asks:Vec<[Decimal;2]> = depth.updated_asks.iter()
                .map(|(price, qty)| [*price, *qty])
                .collect();

                DepthUpdate::from_value(bids, asks)
            },
            None => {
                println!("no depth to update to wss!");
                DepthUpdate::new()
            }
        };

        let message = WsMessage::Depth { depth: depth_update };

        self.publish_to_ws(&channel, conn, message);
    }

}
