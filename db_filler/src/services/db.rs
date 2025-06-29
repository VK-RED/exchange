use common::message::db_filler::DbFillerMessage;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct DbManager{
    conn: Pool<Postgres>
}

impl DbManager{
    pub async fn new() -> Self{
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let conn = PgPoolOptions::new().
        max_connections(5)
        .connect(&db_url)
        .await
        .expect("Error while gettting DB Connection")
        ;

        println!("DB connection successfull");

        Self {
            conn
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

            },
            DbFillerMessage::AddTrade(trades) => {

            },
            DbFillerMessage::UpdateCancelOrders(orders) => {

            }
        }

    }


}