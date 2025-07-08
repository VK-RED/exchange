use serde::{Deserialize, Serialize};

use crate::types::{order::{OrderSide, OrderType, Price, Quantity}};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum MessageFromApi{
    CreateOrder(CreateOrderPayload),
    CancelOrder(CancelOrderPayload),
    CancelAllOrders(CancelOrdersPayload),
    GetAllOpenOrders(OpenOrdersPayload),
    GetDepth(String),
}

impl MessageFromApi {
    pub fn get_market(&self) -> &str{
        match self{
            MessageFromApi::CreateOrder(order) => &order.market,
            MessageFromApi::CancelOrder(order) => &order.market,
            MessageFromApi::CancelAllOrders(order) => &order.market,
            MessageFromApi::GetAllOpenOrders(order) => &order.market,
            MessageFromApi::GetDepth(order) => order
        }
    }

    pub fn get_channel_to_publish(&self)->String{
        match self{
            MessageFromApi::CreateOrder(order) => order.id.clone(),
            MessageFromApi::CancelOrder(order) => order.order_id.clone(),
            MessageFromApi::CancelAllOrders(order) => order.user_id.clone(), // send message on the users channel
            MessageFromApi::GetAllOpenOrders(order) => order.user_id.clone(), // send message on the users channel
            MessageFromApi::GetDepth(order) => order.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserMessageFromApi {
    // add messages like user create, user deposit
    Balance(String)
}

impl UserMessageFromApi {
    pub fn try_deserialized(serialized:&str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Self>(serialized)
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

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct CancelOrdersPayload {
    pub market: String,
    pub user_id: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct OpenOrdersPayload{
    pub market: String,
    pub user_id: String,
}