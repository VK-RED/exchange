use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    pub id: u32,
    pub market: String,
    pub price: String,
    pub quantity: String,
    pub quote_qty: String,
    pub matched_at: u32,
}