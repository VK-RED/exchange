#[derive(Debug)]
pub enum EngineError {
    OrderBookError(OrderBookError),
    BalanceError(BalanceError),
}

#[derive(Debug)]
pub enum OrderBookError{
    UserNotFound,
}

#[derive(Debug)]
pub enum BalanceError {
    InsufficientBalance
}