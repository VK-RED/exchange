use common::types::error::ErrorResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum EngineError {
    #[error("User Not Found ! Please Signup !")]
    UserNotFound,
    #[error("The order can't be executed, as it will be filled partially !")]
    PartialOrderFill,
    #[error("Given user does not have permission to perform the action")]
    MismatchUser,
    #[error("Enter valid order_id")]
    InvalidOrderId,
    #[error("Internal Server Error")]
    InternalError,
    #[error("User does not have sufficient balance")]
    InsufficientBalance,
    #[error("Please Enter Valid Market")]
    InvalidMarket,
}

impl EngineError {
    pub fn to_error_response(self:Self) -> ErrorResponse{
        let error_code = serde_json::to_string(&self).unwrap_or_else(|_|"INTERNAL_ERROR".to_string());
        let error_message = self.to_string();

        ErrorResponse {
            code: error_code,
            message: error_message
        }
    }
}