use serde::{Deserialize, Serialize};

pub type Price = u64;
pub type Quantity = u16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill{
    pub order_id: String,
    pub quantity: Quantity,
    pub maker_id: String,
    pub price: Price
}

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