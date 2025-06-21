use std::{collections::HashMap, sync::{mpsc, Arc, Mutex}};

use common::message::message_from_api::MessageFromApi;
use r2d2_redis::{r2d2::{self, Pool}, RedisConnectionManager};

use crate::orderbook::OrderBook;

// TODO: SEPARATE IT FROM USER AND ORDER RELATED STUFFS
pub type MarketTx = mpsc::Sender<MessageFromApi>;

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

        let assets = [
            ("SOL".to_string(),9),
            ("BONK".to_string(),8),
            ("JUP".to_string(), 6)
        ];

        let mut orderbooks = vec![];

        for (asset, decimals) in assets {
            orderbooks.push(OrderBook::new(asset.clone(),decimals));
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

    /// sets the initial balance for dummy users
    pub fn set_base_balance(user_balances:Arc<Mutex<UserAssetBalance>>){
        let mut guard = user_balances.lock().unwrap();

        let user_ids = [
            "random1".to_string(),
            "random2".to_string(),
            "random32".to_string(),
        ];

        for user_id in user_ids {
            guard.insert(user_id.clone(), HashMap::new());

            let assets = [
                ("SOL".to_string(),9),
                ("BONK".to_string(),8),
                ("JUP".to_string(), 6),
                ("USDC".to_string(), 6)

            ];

            for (asset, decimal) in assets {

                let base = 10_u64;
                let lamports = base.pow(decimal);

                let amount = 10000 * lamports;

                let asset_balance = guard.get_mut(&user_id).unwrap();

                match asset_balance.get_mut(&asset){
                    Some(_balance) => {
                        println!("wtf");
                    },
                    None => {
                        let balance = AssetBalance{
                            available_amount:amount,
                            locked_amount:0,
                        };

                        let amount_set = amount / lamports;

                        println!("set {} {} for user : {} ", amount_set, asset, user_id);

                        asset_balance.insert(asset, balance);
                    }   
                }

            }

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

    pub fn deserialize_message(message:&str)->Result<MessageFromApi, serde_json::Error>{
        let deserialized = serde_json::from_str::<MessageFromApi>(message);
        deserialized   
    }

    
}

#[derive(Clone, Debug)]
pub struct AssetBalance{
    pub available_amount: u64,
    pub locked_amount: u64,
}

impl AssetBalance {
    pub fn new() -> Self{
        Self {
            available_amount: 0,
            locked_amount:0
        }
    }


}