use serde::{Deserialize, Serialize};

use crate::types::order::{Fill, Quantity};

pub enum MessageFromEngine{
    OrderPlaced(OrderPlacedResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPlacedResponse {
    pub order_id: String,
    pub executed_quantiy: Quantity,
    pub fills: Vec<Fill>
}