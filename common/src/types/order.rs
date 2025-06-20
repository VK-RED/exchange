use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Copy, Serialize, PartialEq, PartialOrd)]
pub enum OrderSide{
    Buy,
    Sell
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum OrderType{
    Limit,
    Market,
}

pub type Price = u64;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: u16,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Order {
    pub fn get_opposing_side(&self) -> OrderSide{
        match self.side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }
}


#[derive(Deserialize, Debug, Clone, Serialize, PartialEq)]
pub enum MessageType{
    CreateOrder(Order)
}