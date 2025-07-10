use std::time::Instant;
use actix_web::{delete, web::{Data, Json}, Responder};
use common::{message::{api::{CancelOrderPayload, MessageFromApi}, engine::OrderCancelledResponse}};
use serde::{Deserialize, Serialize};

use crate::{entrypoint::AppState, services::redis::{PubSubService, RedisService}, utils::{engine_res_wrapper::get_engine_http_response, observer::Observer}};

#[derive(Deserialize, Serialize)]
pub struct CancelOrder{
    pub user_id: String,
    pub order_id: String,
    pub market: String
}

#[delete("/order")]
pub async fn cancel_order(app_state:Data<AppState> ,json:Json<CancelOrder>) -> impl Responder{

    let now = Instant::now();
    let route = String::from("Cancel Single Order");
    
    let observer = Observer::new(now, route);

    let guard = &app_state.redis_pool;
    let conn_1 = guard.get().unwrap();
    let mut conn_2 = guard.get().unwrap();

    let mut redis_service = RedisService::new(conn_1);

    let payload = json.0;
    let channel_to_subscribe = payload.order_id.clone();

    let pub_sub =  conn_2.as_pubsub();
    let mut pub_sub_service = PubSubService::new(pub_sub, &channel_to_subscribe);

    let cancel_order_payload = CancelOrderPayload {
        market: payload.market,
        order_id: payload.order_id,
        user_id: payload.user_id,
    };

    let message_from_api = MessageFromApi::CancelOrder(cancel_order_payload);

    get_engine_http_response::<OrderCancelledResponse>(
        message_from_api, 
        &mut redis_service, 
        &mut pub_sub_service,
        observer
    )

}