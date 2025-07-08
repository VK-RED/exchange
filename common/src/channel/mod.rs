
// orders from api to engine are pushed in this channel
pub const ORDER_CHANNEL: &'static str = "orders";

// db updated from db_filler are pushed in this channel
pub const DB_CHANNEL:&'static str = "db_filler";

// user queries are pushed in this channel
pub const USER_CHANNEL: &'static str = "user";