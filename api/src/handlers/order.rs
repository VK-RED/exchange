use actix_web::{post, web::{Data, Json}, HttpResponse, Responder};
use common::types::order::{MessageType, Order, OrderSide};
use r2d2_redis::redis::{Commands};
use serde::Deserialize;
use uuid::Uuid;

use crate::{entrypoint::AppState, errors::{CustomApiError}};

#[derive(Deserialize, Debug)]
pub struct CreateOrder{
    pub user_id: String,
    pub side: OrderSide,
    pub market: String,
    pub price: u64,
    pub quantity: u16,
}

pub type RedisCustomResult<T> =  Result<T, r2d2_redis::redis::RedisError>;

#[post("/order")]
async fn create_order(payload:Json<CreateOrder>, state:Data<AppState>) -> impl Responder{

    let id = Uuid::new_v4().to_string();

    let order = Order {
        id:id.clone(),
        market: payload.market.clone(),
        price: payload.price,
        quantity: payload.quantity,
        side: payload.side,
        user_id: payload.user_id.clone(),
    };

    let message_type = MessageType::CreateOrder(order);

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

    println!("received message from channel: {:?}, message:{:?}", id, message);

    // TODO: DESERIALIZE THE MESSAGE AND SEND THE CREATE ORDER RESPONSE TO THE CLIENT
    HttpResponse::Ok().json("Order Created Successfully !")

}