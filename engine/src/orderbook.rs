use std::{sync::{Arc, Mutex}};

use common::types::order::{MessageType, Order, OrderSide};
use r2d2_redis::{r2d2::{Pool}, redis::Commands, RedisConnectionManager};
use crate::engine::UserAssetBalance;

pub type RedisResponse = Result<(), r2d2_redis::redis::RedisError>;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub market: String,
    pub bids: Vec<Order>,    
    pub asks: Vec<Order>,
    pub last_price: u128,
}

impl OrderBook {

    pub fn new(market:String) -> Self {
        Self { 
            market, 
            last_price:0,
            bids: vec![], 
            asks:vec![]
        }
    }

    pub fn add_order(&mut self, order:Order){
        let orders = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        }; 
        orders.push(order);
    }

    pub fn process(
            &mut self, 
            message_type:MessageType, 
            user_balances:Arc<Mutex<UserAssetBalance>>,
            pool:&Pool<RedisConnectionManager>
    ){
        let mut conn = pool.get().unwrap();

        match message_type {

            MessageType::CreateOrder(order) => {

                let order_id = order.id.clone();

                self.add_order(order);

                let publish_message = "Order completed Successfully";
                let redis_response:RedisResponse = conn.publish(&order_id, publish_message);

                println!("published message to order id : {}", order_id);

                if let Err(e) = redis_response {
                    println!("Error while publishing to the order id : {}", e);
                }
            },
        }

    }

}
