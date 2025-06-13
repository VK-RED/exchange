use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Copy, Serialize, PartialEq, PartialOrd)]
pub enum OrderSide{
    Buy,
    Sell
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub price: u128,
    pub quantity: u16,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq)]
pub enum MessageType{
    CreateOrder(Order)
}