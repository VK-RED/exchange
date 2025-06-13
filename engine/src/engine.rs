use std::{collections::HashMap, sync::mpsc};

use common::types::order::MessageType;
use r2d2_redis::{r2d2::Pool, RedisConnectionManager};

use crate::orderbook::OrderBook;

pub type UserBalance = HashMap<String, u128>;
pub type MarketTx = mpsc::Sender<MessageType>;

#[derive(Clone)]
pub struct Engine {
    pub orderbooks: Vec<OrderBook>,
    pub user_balances: HashMap<String, UserBalance>,
    pub redis_pool: Pool<RedisConnectionManager>,
    pub order_queue_key: String,
}

impl Engine {

    // TODO: RECOVER SNAPSHOT 
    // AND PERIODICALLY SAVE THE SNAPSHOT
    pub fn init(redis_pool: Pool<RedisConnectionManager>) -> Self{

        let markets = [
            "SOL_USDC".to_string(),
            "BONK_USDC".to_string(),
            "JUP_USDC".to_string()
        ];

        let mut orderbooks = vec![];

        for market in markets {
            orderbooks.push(OrderBook::new(market.clone()));
        }

        // Initially all the balances will be zero
        let balances: HashMap<String, UserBalance> = HashMap::new();

        // should be same as what we push in the queue in api
        let order_queue_key = "orders".to_string();

        Self { 
            orderbooks, 
            user_balances: balances,
            redis_pool, 
            order_queue_key,
        }
    }


    pub fn init_market_tx() -> HashMap<String, MarketTx>{
        HashMap::new()
    }

    pub fn deserialize_message(message:&str)->Result<MessageType, serde_json::Error>{
        let deserialized = serde_json::from_str::<MessageType>(message);
        deserialized   
    }
}