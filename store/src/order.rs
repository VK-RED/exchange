use common::{message::db_filler::OrderStatus, types::order::OrderSide};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Order{
    pub id: String,
    pub quantity: String, 
    pub filled_quantity: String,
    pub price: String,
    pub side: OrderSide,
    pub order_status: OrderStatus,
}