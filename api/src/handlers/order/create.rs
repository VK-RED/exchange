use actix_web::{post, web::{Data, Json}, HttpResponse, Responder};
use common::{
    message::{
        api::{
            CreateOrderPayload, 
            MessageFromApi
        }, engine::OrderPlacedResponse, 
    }, 
    types::{error::ErrorResponse, order::{
        OrderSide, 
        OrderType, 
        Price, Quantity
    }}};
use r2d2_redis::redis::{Commands};
use serde::Deserialize;
use uuid::Uuid;

use crate::{entrypoint::AppState, errors::{CustomApiError}};

#[derive(Deserialize, Debug)]
pub struct CreateOrder{
    pub user_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub market: String,
    pub price: Price,
    pub quantity: Quantity
}

pub type RedisCustomResult<T> =  Result<T, r2d2_redis::redis::RedisError>;
pub type OrderPlacedResult = Result<OrderPlacedResponse, ErrorResponse>;

#[post("/order")]
async fn create_order(payload:Json<CreateOrder>, state:Data<AppState>) -> impl Responder{

    let id = Uuid::new_v4().to_string();

    let order = CreateOrderPayload {
        id:id.clone(),
        market: payload.market.clone(),
        price: payload.price,
        quantity: payload.quantity,
        side: payload.side,
        user_id: payload.user_id.clone(),
        order_type: payload.order_type,
    };

    let message_type = MessageFromApi::CreateOrder(order);

    println!("order created : {:?}", message_type);

    let serialized = serde_json::to_string(&message_type);

    if serialized.is_err() {
        return CustomApiError::internal_error();
    }

    let serialized = serialized.unwrap();

    // We need two connections as we cant borrow them as mutable twice !
    
    let pub_sub = state.redis_pool.get();
    let conn = state.redis_pool.get();

    if conn.is_err() || pub_sub.is_err(){
        return CustomApiError::internal_error();
    }

    let mut conn = conn.unwrap();
    let mut pub_sub = pub_sub.unwrap();
    let mut pub_sub = pub_sub.as_pubsub();

    let res = pub_sub.subscribe(id.clone());

    if res.is_err(){
        return CustomApiError::internal_error();
    }

    let _: RedisCustomResult<()> = conn.lpush("orders", serialized);

    let message = pub_sub.get_message();
    let res = pub_sub.unsubscribe(id.clone());

    if res.is_err(){
        return CustomApiError::internal_error();
    }

    let mut result: OrderPlacedResult = Err(ErrorResponse{
        code:"INTERNAL_ERROR".to_string(),
        message:"Internal Server Error".to_string()
    });

    if let Ok(data) = message {
        let payload:RedisCustomResult<String> = data.get_payload();

        if let Ok(val) = payload {
            let deserialized: Result<OrderPlacedResult, serde_json::Error> = serde_json::from_str(&val);
            
            if let Ok(deserial_data) = deserialized {
                result = deserial_data;
            }
        }
    }

    match result {
        Ok(val) => {
            HttpResponse::Ok().json(val)
        },
        Err(e) => {
            HttpResponse::BadRequest().json(e)
        }
    }

    

}