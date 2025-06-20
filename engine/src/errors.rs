#[derive(Debug)]
pub enum EngineError {
    OrderBookError(OrderBookError),
    BalanceError(BalanceError),
}

#[derive(Debug)]
pub enum OrderBookError{
    UserNotFound,
    ExecuteMarketOrder,
}

#[derive(Debug)]
pub enum BalanceError {
    InsufficientBalance
}