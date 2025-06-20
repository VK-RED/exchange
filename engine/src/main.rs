use std::{sync::{mpsc, Arc, Mutex}, thread};
use common::types::order::{MessageType};
use r2d2_redis::{redis::{Commands, RedisError}};

use crate::{engine::{Engine}};

mod orderbook;
mod engine;
mod errors;

// TOTAL THREADS = 1 MAIN + (1* NO.OF.ORDERBOOKS ) + 1 USER REQ thread 

fn main() {

    println!("Starting the engine");

    let engine = Engine::init();
    let mut markets_tx = Engine::init_market_tx();
    let user_balances = engine.user_balances;

    let user_balances = Arc::new(Mutex::new(user_balances));

    Engine::set_base_balance(Arc::clone(&user_balances));

    for mut orderbook in engine.orderbooks {

        let (tx, rx) = mpsc::channel::<MessageType>();

        markets_tx.insert(orderbook.market.clone(), tx);

        let pool_clone = engine.redis_pool.clone();
        let user_balances_clone = Arc::clone(&user_balances);

        println!("Spawning thread for the orderbook : {:?}", &orderbook.market);

        thread::spawn(move||{
            loop{
                let message = rx.recv();

                match message {

                    Ok(message_type) => {

                        orderbook.process(
                            message_type, 
                            user_balances_clone.clone(),
                            &pool_clone
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
    let mut conn = pool.get().unwrap();
    let queue_key = &engine.order_queue_key;

    loop {

        let res: Result<Option<String>,RedisError> = conn.rpop(queue_key);

        match res {

            Ok(message_res) => {

                if let Some(message) = message_res {

                    println!("--------------------------------------------------------");
                    println!("received message : {}", message);

                    let deserialized = Engine::deserialize_message(&message);

                    match deserialized {

                        Ok(message_type) => {

                            match message_type {

                                MessageType::CreateOrder(order) => {
                                    let market = &order.market;
                                    let tx_res = markets_tx.get(market);

                                    match tx_res {
                                        Some(tx) => {

                                            let tx_send_err = format!("Error while sending order to the orderbook : {}",market);
                                            
                                            tx.send(MessageType::CreateOrder(order))
                                            .expect(&tx_send_err);
                                        },
                                        None => {
                                            println!("No tx found for the market : {}", market);
                                        }
                                    }

                                },
                            }

                        },
                        Err(e) => {
                            println!("Error while deserializing message : {}, error : {}", message, e);
                        }
                    }
                }

            },  
            Err(e) => {
                println!("Error while polling from the queue : {}", &e);
            }
        }

    }
    
}
