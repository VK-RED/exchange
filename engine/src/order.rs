use common::{message::api::CreateOrderPayload, types::order::{OrderSide, OrderType}};
use serde::{Deserialize, Serialize};

pub type Price = u64;
pub type Quantity = u16;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
}

impl Order {

    pub fn from_create_order_payload(payload: CreateOrderPayload) -> Self {
        Self { 
            id: payload.id, 
            user_id: payload.user_id, 
            side: payload.side, 
            market: payload.market,
            order_type: payload.order_type, 
            price: payload.price, 
            quantity: payload.quantity, 
        }
    }

    pub fn get_opposing_side(&self) -> OrderSide{
        match self.side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}