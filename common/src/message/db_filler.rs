use std::{fmt::Display};

use serde::{Deserialize, Serialize};

use crate::types::order::{OrderSide, Price, Quantity};

/// Message from engine to db filler
#[derive(Serialize, Deserialize)]
pub enum DbFillerMessage{
    AddTrade(Vec<Trade>),
    AddAndUpdateOrders{
        add_order: Option<AddOrderToDb>,
        update_orders: Vec<UpdateOrder>,
    },
    UpdateCancelOrders(Vec<String>)
}

impl DbFillerMessage {
    pub fn get_deserialized(message:&str) -> Option<DbFillerMessage>{
        let res = serde_json::from_str(message)
        .map_or(None, |val:DbFillerMessage|{
            Some(val)
        });

        res
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum OrderStatus{
    Open,
    Filled,
    Cancelled,
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::Filled => write!(f,"Filled"),
            Self::Cancelled => write!(f, "Cancelled")
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    pub id: u32,
    pub market: String,
    pub price: Price,
    pub quantity: Quantity,
    pub quote_qty: Quantity,
    pub timestamp: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddOrderToDb{
    pub order_id: String,
    pub quantity: Quantity, 
    pub filled_quantity: Quantity,
    pub price: Price,
    pub side: OrderSide,
    pub status: OrderStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateOrder{
    pub order_id: String,
    pub filled_quantity:Quantity,
    pub status: OrderStatus, 
}
