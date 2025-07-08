use actix_web::{get, web::{Data, Path}, Responder};
use common::message::{api::UserMessageFromApi, engine::UserBalanceResponse};

use crate::{entrypoint::AppState, services::redis::{PubSubService, RedisService}, utils::engine_res_wrapper::get_user_engine_http_response};

#[get("user/{user_id}/balance")]
pub async fn get_user_balance(app_state: Data<AppState>, path: Path<String>) -> impl Responder{
    let user_id = path.into_inner();

    let guard = &app_state.redis_pool;
    let conn_1 = guard.get().unwrap();
    let mut conn_2 = guard.get().unwrap();

    let mut redis_service = RedisService::new(conn_1);
    let channel_to_subscribe = format!("{}-balance", user_id);

    let pub_sub =  conn_2.as_pubsub();
    let mut pub_sub_service = PubSubService::new(pub_sub, &channel_to_subscribe);


    let user_message = UserMessageFromApi::Balance(user_id);

    get_user_engine_http_response::<UserBalanceResponse>(
        channel_to_subscribe.clone(), 
        user_message, 
        &mut redis_service, 
        &mut pub_sub_service
    )
}