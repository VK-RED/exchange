use common::{message::api::CreateOrderPayload, types::order::{OrderSide, OrderType, Price, Quantity}};
use rust_decimal::dec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub filled:Quantity,
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
            filled: dec!(0), 
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

#[derive(Debug, Clone)]
pub struct OrdersWithQuantity{
    pub orders: Vec<Order>,
    pub total_quantity: Quantity,
}

impl OrdersWithQuantity {
    pub fn new(total_quantity: Quantity, orders: Vec<Order>) -> Self {
        Self { orders, total_quantity }
    }
}