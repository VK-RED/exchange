use serde::{Deserialize, Serialize};

use crate::types::order::{OrderSide, OrderType, Price, Quantity};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum MessageFromApi{
    CreateOrder(CreateOrderPayload)
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateOrderPayload {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
}