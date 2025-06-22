use serde::{Deserialize, Serialize};

use crate::types::order::{Price, Quantity};

pub enum MessageFromEngine{
    OrderPlaced(OrderPlacedResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPlacedResponse {
    pub order_id: String,
    pub executed_quantity: Quantity,
    pub fills: Vec<OrderFill>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderFill{
    pub price: Price,
    pub quantity: Quantity,
    pub trade_id: u32,
}