use actix_web::{web::{Data, Json}, HttpResponse, get};
use common::message::{api::{MessageFromApi, OpenOrdersPayload}, engine::AllOpenOrdersResponse};
use serde::Deserialize;

use crate::{entrypoint::AppState, errors::CustomApiError, services::redis::{PubSubService, RedisService}, utils::engine_res_wrapper::get_engine_http_response};

#[derive(Deserialize)]
pub struct OpenOrders{
    pub user_id: String,
    pub market: String,
}

#[get("/order/all")]
pub async fn get_all_open_orders(
    app_state:Data<AppState>, 
    json: Json<OpenOrders>
) -> HttpResponse {

    let pool = &app_state.redis_pool;
    let conn_1_res = pool.get();
    let conn_2_res = pool.get();

    if let Err(e) = conn_1_res {
        println!("error while getting redis connection from pool :{}",e);
        return CustomApiError::internal_error();
    }

    if let Err(e) = conn_2_res {
        println!("error while getting redis connection from pool :{} ",e);
        return CustomApiError::internal_error();
    }

    let conn_1 = conn_1_res.unwrap();
    let mut conn_2 = conn_2_res.unwrap();

    let mut redis_service = RedisService::new(conn_1);
    
    let channel_to_publish = json.0.user_id.clone();
    let pub_sub = conn_2.as_pubsub();

    let mut pub_sub_service = PubSubService::new(
        pub_sub, 
        &channel_to_publish
    );

    let message_from_api = MessageFromApi::GetAllOpenOrders(OpenOrdersPayload{
        market: json.0.market,
        user_id: json.0.user_id
    });

    get_engine_http_response::<AllOpenOrdersResponse>(
        message_from_api, 
        &mut redis_service, 
        &mut pub_sub_service
    )

}