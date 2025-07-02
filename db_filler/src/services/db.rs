use common::message::db_filler::DbFillerMessage;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use store::{Order, UpdateDbOrder};

pub struct DbManager{
    pool: Pool<Postgres>
}

impl DbManager{
    pub async fn new() -> Self{
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPoolOptions::new().
        max_connections(5)
        .connect(&db_url)
        .await
        .expect("Error while gettting DB Connection")
        ;

        println!("DB connection successfull");

        Self {
            pool
        }
    }

    pub async fn process_message(&self, message:&str){
        
        let message_res =  DbFillerMessage::get_deserialized(message);
        
        if let None = message_res {
            println!("cannot deserialize messgae : {}", message);
            return;
        }

        let filler_message = message_res.unwrap();

        match filler_message {

            DbFillerMessage::AddAndUpdateOrders { 
                add_order, 
                update_orders 
            } => {

                let mut add_res = None;
                let update_res;

                let update_db_orders: Vec<UpdateDbOrder> = update_orders.into_iter().map(|order|{

                    UpdateDbOrder { 
                        order_id: order.order_id, 
                        filled_quantity:order.filled_quantity.to_string(), 
                        status: order.status.to_string(), 
                    }
                }).collect();

                if let Some(order) = add_order {

                    let parsed_order = Order{
                        filled_quantity: order.filled_quantity.to_string(),
                        id: order.order_id,
                        order_status: order.status.to_string(),
                        price: order.price.to_string(),
                        quantity: order.quantity.to_string(),
                        side:order.side.to_string(),
                    };

                    let res = Order::add_order(parsed_order, &self.pool);
                    add_res = Some(res);
                }
                else{
                    println!("add_order is NONE, only update orders will be executed");
                }

                update_res = Order::update_orders(update_db_orders, &self.pool);


                if let Some(val) = add_res {
                    let res = val.await;

                    if let Err(e) = res {
                        println!("error while adding order to db : {}", e);
                    }
                } 

                if let Err(e) = update_res.await {
                    println!("error while updating orders: {}", e);
                }

            },
            DbFillerMessage::AddTrade(trades) => {

                let parsed_trades: Vec<store::Trade> = trades.into_iter().map(|trade| {

                    let add_trade = store::Trade {
                        id : i64::from(trade.id),
                        market: trade.market,
                        matched_at: i64::from(trade.timestamp),
                        price: trade.price.to_string(),
                        quantity: trade.quantity.to_string(),
                        quote_qty: trade.quote_qty.to_string(),
                    };

                    add_trade

                }).collect();

                let res = store::Trade::add_trades(parsed_trades, &self.pool).await;

                if let Err(e) = res{
                    println!("Error while adding trade : {}", e);
                }

            },
            DbFillerMessage::UpdateCancelOrders(orders) => {

                let res = Order::update_cancelled_orders(orders, &self.pool).await;
                if let Err(e) = res {
                    println!("Error while cancelling orders: {}", e);
                }
            }
        }

    }


}