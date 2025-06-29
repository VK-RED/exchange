use dotenv::dotenv;

use crate::services::redis::RedisService;

mod services;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let mut redis_service = RedisService::new().await;

    println!("starting db_filler");

    loop {
        let message = redis_service.get_message_from_engine().await;
        println!("recieved message from engine : {:?}", message );
    
    }
    
}
