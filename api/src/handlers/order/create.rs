use std::time::Instant;

use actix_web::{post, web::{Data, Json}, Responder};
use common::{
    message::{
        api::{
            CreateOrderPayload, 
            MessageFromApi
        }, engine::OrderPlacedResponse, 
    }, 
    types::{order::{
        OrderSide, 
        OrderType, 
        Price, Quantity
    }}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{entrypoint::AppState, services::redis::{PubSubService, RedisService}, utils::{engine_res_wrapper::get_engine_http_response, observer::Observer}};

#[derive(Deserialize, Debug)]
pub struct CreateOrder{
    pub user_id: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub market: String,
    pub price: Price,
    pub quantity: Quantity
}

#[post("/order")]
async fn create_order(payload:Json<CreateOrder>, state:Data<AppState>) -> impl Responder{

    let now = Instant::now();
    let route = String::from("Place new Order");
    let observer = Observer::new(now, route);

    let guard = &state.redis_pool;
    let conn_1 = guard.get().unwrap();
    let mut conn_2 = guard.get().unwrap();

    let mut redis_service = RedisService::new(conn_1);

    let id = Uuid::new_v4().to_string();
    let pub_sub =  conn_2.as_pubsub();
    let mut pub_sub_service = PubSubService::new(pub_sub, &id);

    let order = CreateOrderPayload {
        id:id.clone(),
        market: payload.market.clone(),
        price: payload.price,
        quantity: payload.quantity,
        side: payload.side,
        user_id: payload.user_id.clone(),
        order_type: payload.order_type,
    };

    let message_from_api = MessageFromApi::CreateOrder(order);

    get_engine_http_response::<OrderPlacedResponse>(
        message_from_api, 
        &mut redis_service, 
        &mut pub_sub_service,
        observer
    )

    

}