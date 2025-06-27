use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{order::{OrderSide, Price, Quantity}};

#[derive(Serialize, Deserialize)]
pub enum MessageFromEngine{
    OrderPlaced(OrderPlacedResponse),
    OrderCancelled(OrderCancelledResponse),
    AllOrdersCancelled(OrdersCancelledResponse),
    AllOpenOrders(AllOpenOrdersResponse),
    GetDepth(DepthResponse)
}

type EngineResult<T> = Result<T, ()>;

impl MessageFromEngine{
    pub fn serialize_data_as_ok(&self)->String{
        let err_msg = String::from("INTERNAL_ERROR");
        match self{
            MessageFromEngine::OrderCancelled(data) => {
                let ok_data: EngineResult<&OrderCancelledResponse> = Ok(data);
                serde_json::to_string(&ok_data).unwrap_or_else(|_|err_msg)
            } ,
            MessageFromEngine::OrderPlaced(data) => {
                let ok_data: EngineResult<&OrderPlacedResponse> = Ok(data);
                serde_json::to_string(&ok_data).unwrap_or_else(|_|err_msg)
            },
            MessageFromEngine::AllOrdersCancelled(data) => {
                let ok_data: EngineResult<&OrdersCancelledResponse> = Ok(data);
                serde_json::to_string(&ok_data).unwrap_or_else(|_|err_msg)
            },
            MessageFromEngine::AllOpenOrders(data) => {
                let ok_data: EngineResult<&AllOpenOrdersResponse> = Ok(data);
                serde_json::to_string(&ok_data).unwrap_or_else(|_|err_msg)
            },
            MessageFromEngine::GetDepth(data) => {
                let ok_data: EngineResult<&DepthResponse> = Ok(data);
                serde_json::to_string(&ok_data).unwrap_or_else(|_|err_msg)
            },
        }   
    }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelledResponse {
    pub order_id: String,
    pub quantity: Quantity,
    pub executed_quantity: Quantity,
    pub side: OrderSide,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelAllOrders {
    pub order_id: String,
    pub quantity: Quantity,
    pub executed_quantity: Quantity,
    pub side: OrderSide,
    pub price: Price,
}

pub type OrdersCancelledResponse = Vec<CancelAllOrders>;

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenOrder{
    pub order_id: String,
    pub quantity: Quantity,
    pub executed_quantity: Quantity,
    pub side: OrderSide,
    pub price: Price,
}

pub type AllOpenOrdersResponse = Vec<OpenOrder>;

#[derive(Serialize, Deserialize, Debug)]
/// first element is price and
/// second element is quantity 
pub struct DepthResponse{
    pub bids: Vec<[Decimal;2]>,
    pub asks: Vec<[Decimal;2]>
}