use std::{sync::{mpsc, Arc, Mutex}, thread};
use common::message::{api::MessageFromApi};

use crate::{engine::Engine, services::redis::RedisService,errors::EngineError};

mod orderbook;
mod engine;
mod errors;
mod order;
mod services;

// TOTAL THREADS = 1 MAIN + (1* NO.OF.ORDERBOOKS ) + 1 USER REQ thread 

fn main() {

    println!("Starting the engine");

    let engine = Engine::init();
    let mut markets_tx = Engine::init_market_tx();
    let user_balances = engine.user_balances;

    let user_balances = Arc::new(Mutex::new(user_balances));

    Engine::set_base_balance(Arc::clone(&user_balances));

    for mut orderbook in engine.orderbooks {

        let (tx, rx) = mpsc::channel::<MessageFromApi>();

        markets_tx.insert(orderbook.market.clone(), tx);

        let pool_clone = engine.redis_pool.clone();
        let user_balances_clone = Arc::clone(&user_balances);

        let redis_service = RedisService::new(pool_clone);

        println!("Spawning thread for the orderbook : {:?}", &orderbook.market);

        thread::spawn(move||{
            loop{
                let message = rx.recv();

                match message {

                    Ok(message_type) => {

                        orderbook.process(
                            message_type, 
                            user_balances_clone.clone(),
                            &redis_service
                        );
                    },
                    Err(e) => {
                        println!("Error when receiving message from main : {:?}", e);
                    },
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

    let pool = engine.redis_pool.clone();
    let redis = RedisService::new(pool);

    loop {

        let res = redis.get_message_from_api();

        match res {

            Some(message_res) => {

                if let Some(message) = message_res {

                    println!("--------------------------------------------------------");
                    println!("received message : {}", message);

                    let deserialized = Engine::deserialize_message(&message);

                    match deserialized {

                        Ok(message_type) => {

                            let market = message_type.get_market();
                            let channel_to_publish = message_type.get_channel_to_publish();

                            let tx_res = markets_tx.get(market);

                            match tx_res {
                                None => {
                                    println!("No tx found for the market : {}", market);
                                    redis.publish_message_to_api(channel_to_publish, Err(EngineError::InvalidMarket));
                                },
                                Some(tx) => {
                                    let tx_send_err = format!("Error while sending order to the orderbook : {}",market);
                                    tx.send(message_type).expect(&tx_send_err);
                                }
                            }

                        },
                        Err(e) => {
                            println!("Error while deserializing message : {}, error : {}", message, e);
                        }
                    }
                }

            },  
            None => {},
        }

    }
    
}
