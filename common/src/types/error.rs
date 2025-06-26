use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn serialize_as_err(self) -> String {
        let err: Result<(), ErrorResponse> = Err(self);
        serde_json::to_string(&err).unwrap_or_else(|_|"INTERNAL_ERRROR".to_string())
    }
}