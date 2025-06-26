use actix_web::{delete, web::{Data, Json}, HttpResponse, Responder, ResponseError};
use common::{message::{api::{CancelOrderPayload, MessageFromApi}, engine::OrderCancelledResponse}, types::error::ErrorResponse};
use serde::{Deserialize, Serialize};

use crate::{entrypoint::AppState, services::redis::{PubSubService, RedisService}};

#[derive(Deserialize, Serialize)]
pub struct CancelOrder{
    pub user_id: String,
    pub order_id: String,
    pub market: String
}

pub type CancelOrderResult = Result<OrderCancelledResponse, ErrorResponse>;

#[delete("/order")]
pub async fn cancel_order(app_state:Data<AppState> ,json:Json<CancelOrder>) -> impl Responder{

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

    let res = pub_sub_service.subscribe();

    if res.is_err(){
        return res.unwrap_err().error_response();
    }
    
    let res = redis_service.publish_message_to_engine(message_from_api);

    if res.is_err(){
        return res.unwrap_err().error_response();
    }

    let message = pub_sub_service.get_message_from_engine();

    let res = pub_sub_service.unsubscribe();

    if res.is_err(){
        return res.unwrap_err().error_response();
    }

    match message {
        Ok(msg) => {
            let deserialized: Result<CancelOrderResult, serde_json::Error> = serde_json::from_str(&msg);
            match deserialized{
                Ok(val) => {
                    match val {
                        Ok(res) => {
                            HttpResponse::Ok().json(res)
                        },
                        Err(e) => {
                            HttpResponse::BadGateway().json(e)
                        }
                    }
                },
                Err(e) => {
                    println!("Error while deserialising engine message");
                    e.error_response()
                }
            }
        },
        Err(e) => {
            e.error_response()
        }
    }

}