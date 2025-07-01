use dotenv::dotenv;

use crate::services::{db::DbManager, redis::RedisService};

mod services;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let mut redis_service = RedisService::new().await;
    let db_manager = DbManager::new().await;

    println!("starting db_filler");

    loop {
        let message = redis_service.get_message_from_engine().await;
        if let Some(msg) = message {
            println!("received msg : {}", msg);
            db_manager.process_message(&msg).await;
        }        
    }
    
}
