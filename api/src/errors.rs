use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::derive::{Display, Error};
use serde::{Serialize, Deserialize};

#[derive(Display, Error, Debug)]
pub enum ApiError{
    #[display("Unauthorized")]
    UnAuthorized,
    #[display("Internal Server Error")]
    InternalServerError,
}

#[derive(Display, Error, Debug, Serialize, Deserialize)]
#[display("{}", error)]
pub struct CustomApiError{
    pub error: String,
}


impl ResponseError for CustomApiError{}

impl ResponseError for ApiError{
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self{
            ApiError::UnAuthorized => StatusCode::UNAUTHORIZED,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

