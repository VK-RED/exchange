use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::order::{Price, Quantity};

#[derive(Debug, Serialize, Deserialize)]
pub enum WsMessage{
    Trade(Vec<TradeUpdate>),
    Depth{
        depth: DepthUpdate
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeUpdate {
    pub e: String,
    pub t: u32,
    pub p: Price,
    pub q: Quantity,
    pub s: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthUpdate{
    pub bids: Vec<[Decimal;2]>,
    pub asks: Vec<[Decimal;2]>
}

impl DepthUpdate {
    pub fn new() -> Self {
        Self { bids: vec![], asks: vec![] }
    }

    pub fn from_value(bids: Vec<[Decimal; 2]>, asks:Vec<[Decimal; 2]>) -> Self{
        Self { bids, asks }
    }
}