use common::{message::db_filler::OrderStatus, types::order::OrderSide};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AddOrderToDb{
    pub order_id: String,
    pub quantity: String, 
    pub filled_quantity: String,
    pub price: String,
    pub side: OrderSide,
    pub status: OrderStatus,
}