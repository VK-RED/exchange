use actix_web::{HttpResponse, ResponseError};
use common::{message::api::MessageFromApi, types::error::ErrorResponse};
use serde::{de::DeserializeOwned, Serialize};

use crate::services::redis::{PubSubService, RedisService};

pub type MessageResult<T> = Result<T, ErrorResponse>;

/// This is a HTTP Response Wrapper around
/// the message received from engine
pub fn get_engine_http_response<T:DeserializeOwned+Serialize>(
    message_from_api: MessageFromApi,
    redis_service: &mut RedisService,
    pub_sub_service: &mut PubSubService,
) -> HttpResponse{

    if let Err(e) = pub_sub_service.subscribe() {
        return e.error_response();
    }

    if let Err(e) = redis_service.publish_message_to_engine(message_from_api){
        return e.error_response();
    }

    let message = match pub_sub_service.get_message_from_engine() {
        Ok(msg) => msg,
        Err(e) => return e.error_response(),
    };

    if let Err(e) = pub_sub_service.unsubscribe(){
        return e.error_response();
    }

    let deserialized: Result<MessageResult<T>, serde_json::Error> = serde_json::from_str(&message);

    match deserialized {
        Ok(res) => {
            match res {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => HttpResponse::BadRequest().json(e)
            }
        },
        Err(e) =>{
            println!("deserial error : {:?}", e);
            e.error_response()   
        } 
    }
}