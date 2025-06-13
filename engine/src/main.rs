use std::{sync::{mpsc, Arc, Mutex}, thread};
use common::types::order::MessageType;
use r2d2_redis::{r2d2, redis::{Commands, RedisError}, RedisConnectionManager};

use crate::{engine::{Engine}, orderbook::OrderBook};

mod orderbook;
mod engine;

// TOTAL THREADS = 1 MAIN + (1* NO.OF.ORDERBOOKS) + 1 USER REQ thread

#[tokio::main]
async fn main() {

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_e|String::from("redis://127.0.0.1:6379"));
    let manager = RedisConnectionManager::new(redis_url).expect("Failed to create redis manager");
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create Redis Pool");

    println!("Starting the engine");

    let engine = Engine::init(pool);
    let engine = Arc::new(Mutex::new(engine));
    let mut markets_tx = Engine::init_market_tx();

    let markets:Vec<String>;

    // lock will be held if not scoped in a block
    {
        markets  = engine.lock().unwrap()
        .orderbooks.iter()
        .map(|ob| ob.market.clone())
        .collect();
    }


    for market in markets.iter(){
        
        let (tx, rx) = mpsc::channel::<MessageType>();

        println!("Spawning thread for market : {:?}", market);

        markets_tx.insert(market.clone(), tx);

        let engine_clone = Arc::clone(&engine);

        thread::spawn(move||{
            loop {
                let message = rx.recv();
                match message {
                    Err(e) => {
                        println!("Error when receiving message from main : {:?}", e);
                    },
                    Ok(val) => {
                        OrderBook::process_order(val, Arc::clone(&engine_clone));
                    }
                }
            }
        });

        
    }

    println!("Spawning a separate thread for handing non-order-processing requests ...");

    // this is typically to handle things like user balance, current price queries etc..
    thread::spawn(||{
        loop {

        }
    });

    loop {

        let guard = engine.lock().unwrap();
        let pool = &guard.redis_pool;
        let mut conn = pool.get().unwrap();

        let queue_key = &guard.order_queue_key;

        let res: Result<String,RedisError> = conn.rpop(queue_key);

        if res.is_ok(){
            let message = res.unwrap();

            println!("received message : {}", message);

            let deserialized = Engine::deserialize_message(&message);

            if deserialized.is_ok(){

                let message_type = deserialized.unwrap();

                let market = match &message_type{
                    MessageType::CreateOrder(order) => order.market.clone(),
                };

                let market_with_tx = markets_tx.get(&market);
                
                if market_with_tx.is_none(){
                    let err_msg = format!("Cannot find tx for the market : {}", market);
                    println!("{}",err_msg);
                }
                else{
                    let tx = market_with_tx.unwrap();
                    let tx_res = tx.send(message_type);

                    if tx_res.is_err(){
                        println!("error while sending message to orderbook : {}", tx_res.unwrap_err());
                    }
                }
            
            }
            else{
                let err_message = deserialized.unwrap_err();
                println!("Error while deserializing message : {}", err_message);
            }
        }  

    }
    
}
