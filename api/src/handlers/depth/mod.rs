use actix_web::{get, web::{Data, Path}, HttpResponse};
use common::message::{api::MessageFromApi, engine::DepthResponse};

use crate::{entrypoint::AppState, errors::CustomApiError, services::redis::{PubSubService, RedisService}, utils::engine_res_wrapper::get_engine_http_response};

#[get("/depth/{market}")]
pub async fn get_depth(app_state:Data<AppState>, path:Path<String> ) -> HttpResponse{

    let pool = &app_state.redis_pool;
    let conn_1_res = pool.get();
    let conn_2_res = pool.get();

    let market = path.into_inner();

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
    
    let channel_to_publish = market.clone();
    let pub_sub = conn_2.as_pubsub();

    let mut pub_sub_service = PubSubService::new(
        pub_sub, 
        &channel_to_publish
    );

    let message_from_api = MessageFromApi::GetDepth(market);

    get_engine_http_response::<DepthResponse>(
        message_from_api, 
        &mut redis_service, 
        &mut pub_sub_service
    )

}