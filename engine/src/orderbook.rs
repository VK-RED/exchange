use std::{sync::{Arc, Mutex}};

use common::types::order::{MessageType, Order, OrderSide};
use r2d2_redis::redis::Commands;
use crate::engine::Engine;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub market: String,
    pub bids: Vec<Order>,    
    pub asks: Vec<Order>,
    pub current_price: u128,
}

impl OrderBook {

    pub fn new(market:String) -> Self {
        Self { 
            market, 
            current_price:0,
            bids: vec![], 
            asks:vec![]
        }
    }

    pub fn add_order(&mut self, order:Order) -> usize {
        let orders = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        }; 
        orders.push(order);
        orders.len()
    }

    pub fn process_order(message_type:MessageType, engine: Arc<Mutex<Engine>>){

        let mut guard = engine.lock().unwrap();
        let pool = &guard.redis_pool;
        let conn = pool.get();

        if conn.is_err(){
            println!("error while establising redis connection from orderbook");
        }
        else{
            let mut conn = conn.unwrap();
            
            match message_type {
                MessageType::CreateOrder(order) => {
                    let message = "Order processed successfully";
                    let ob = &guard.orderbooks.iter().find(|orderbook| orderbook.market == order.market);

                    if ob.is_none(){
                        let message = format!("No orderbook found for the market : {}", &order.market);
                        let r: Result<(), r2d2_redis::redis::RedisError> = conn.publish(&order.id, message);
                        if r.is_err(){
                            println!("eee : {}", r.unwrap_err());
                        }
                    }

                    let res: Result<(), r2d2_redis::redis::RedisError> = conn.publish(&order.id, message);
                    if res.is_err(){
                        let err = res.unwrap_err();
                        println!("ERROR WHILE PUBLISING TO PUB SUB in market : {}, err: {}", &order.market, err);
                    }
                
                },
            }

        }
        
    }


}
