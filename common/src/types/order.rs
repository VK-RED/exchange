use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Copy, Serialize)]
pub enum OrderSide{
    Buy,
    Sell
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub price: u128,
    pub quantity: u16,
}