use actix_web::{get, web::Data, HttpResponse,ResponseError};
use serde::Serialize;
use store::Trade;

use crate::{entrypoint::AppState, errors::ApiError};

#[derive(Serialize)]
pub struct TradeResponse{
    trades: Vec<Trade>
}

#[get("/trades")]
pub async fn get_trade_history(app_state:Data<AppState>) -> HttpResponse{

    let pool = &app_state.db_pool;
    let try_trades = Trade::get_trades(pool).await;

    match try_trades {
        Ok(trades) => {
            HttpResponse::Ok().json(TradeResponse{trades})
        },
        Err(e)=> {
            println!("error : {} while fetching trades", e);
            ApiError::InternalServerError.error_response()
        }
    }
}