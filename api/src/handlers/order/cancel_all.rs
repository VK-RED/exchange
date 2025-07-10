use std::time::Instant;
use actix_web::{delete, web::{Data, Json}, HttpResponse};
use common::message::{api::{CancelOrdersPayload, MessageFromApi}, engine::OrdersCancelledResponse};
use serde::{Deserialize, Serialize};

use crate::{entrypoint::AppState, services::redis::{PubSubService, RedisService}, utils::{engine_res_wrapper::get_engine_http_response, observer::Observer}};

#[derive(Serialize, Deserialize)]
pub struct CancelAllOrdersPayload {
    pub market: String,
    pub user_id: String,
}

#[delete("/order/all")]
pub async fn cancel_all_orders (app_state:Data<AppState>, json: Json<CancelAllOrdersPayload>) -> HttpResponse {

    let now = Instant::now();
    let route = String::from("Cancell All Orders");
    
    let observer = Observer::new(now, route);

    let guard = &app_state.redis_pool;
    let conn_1 = guard.get().unwrap();
    let mut conn_2 = guard.get().unwrap();

    let mut redis_service = RedisService::new(conn_1);

    let payload = json.0;
    let channel_to_subscribe = payload.user_id.clone();

    let pub_sub =  conn_2.as_pubsub();
    let mut pub_sub_service = PubSubService::new(pub_sub, &channel_to_subscribe);


    let cancel_all_orders_payload = CancelOrdersPayload {
        market: payload.market,
        user_id: payload.user_id
    };

    let message_from_api = MessageFromApi::CancelAllOrders(cancel_all_orders_payload);

    get_engine_http_response::<OrdersCancelledResponse>(
        message_from_api, 
        &mut redis_service, 
        &mut pub_sub_service,
        observer
    )

}