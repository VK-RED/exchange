use std::fmt::Display;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type Price = Decimal;
pub type Quantity = Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill{
    pub order_id: String,
    pub trade_id: u32,
    pub quantity: Quantity,
    pub filled_quantity: Quantity,
    pub maker_id: String,
    pub price: Price
}

#[derive(Deserialize, Debug, Clone, Copy, Serialize, PartialEq, PartialOrd)]
pub enum OrderSide{
    Buy,
    Sell
}

impl Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f,"Buy"),
            Self::Sell => write!(f,"Sell"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum OrderType{
    Limit,
    Market,
}