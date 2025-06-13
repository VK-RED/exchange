use std::{collections::HashMap, sync::mpsc};

use common::types::order::MessageType;
use r2d2_redis::{r2d2::{self, Pool}, RedisConnectionManager};

use crate::orderbook::OrderBook;

pub type MarketTx = mpsc::Sender<MessageType>;

#[derive(Clone, Debug)]
pub struct AssetBalance{
    pub available_amount: u128,
    pub locked_amount: u128,
}

pub type UserAssetBalance = HashMap<String, HashMap<String, AssetBalance>>;

#[derive(Clone)]
pub struct Engine {
    pub orderbooks: Vec<OrderBook>,
    pub user_balances: UserAssetBalance,
    pub redis_pool: Pool<RedisConnectionManager>,
    pub order_queue_key: String,
}


impl Engine {

    // TODO: RECOVER SNAPSHOT 
    // AND PERIODICALLY SAVE THE SNAPSHOT
    pub fn init() -> Self{

        let redis_pool = Self::init_redis_pool();

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
        let balances: UserAssetBalance = HashMap::new();

        // should be same as what we push in the queue in api
        let order_queue_key = "orders".to_string();

        Self { 
            orderbooks, 
            user_balances: balances,
            redis_pool, 
            order_queue_key,
        }
    }

    pub fn init_redis_pool() -> Pool<RedisConnectionManager>{
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_e|String::from("redis://127.0.0.1:6379"));
        let manager = RedisConnectionManager::new(redis_url).expect("Failed to create redis manager");
        let pool = r2d2::Pool::builder().build(manager).expect("Failed to create Redis Pool");
        pool
    }


    pub fn init_market_tx() -> HashMap<String, MarketTx>{
        HashMap::new()
    }

    pub fn deserialize_message(message:&str)->Result<MessageType, serde_json::Error>{
        let deserialized = serde_json::from_str::<MessageType>(message);
        deserialized   
    }

    
}