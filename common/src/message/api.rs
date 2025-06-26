use serde::{Deserialize, Serialize};

use crate::types::{order::{OrderSide, OrderType, Price, Quantity}};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum MessageFromApi{
    CreateOrder(CreateOrderPayload),
    CancelOrder(CancelOrderPayload)
}

impl MessageFromApi {
    pub fn get_market(&self) -> &str{
        match self{
            MessageFromApi::CreateOrder(order) => &order.market,
            MessageFromApi::CancelOrder(order) => &order.market,
        }
    }

    pub fn get_channel_to_publish(&self)->String{
        match self{
            MessageFromApi::CreateOrder(order) => order.id.clone(),
            MessageFromApi::CancelOrder(order) => order.order_id.clone(),
        }
    }
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

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct CancelOrderPayload {
    pub market: String,
    pub order_id: String,
    pub user_id: String,
}