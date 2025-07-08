use std::sync::{Arc};
use dotenv::dotenv;
use tokio::{net::TcpListener, sync::Mutex};
use wss::{handle_connection, AppState, RedisManager, UserManager};

#[tokio::main]
async fn main() {

    dotenv().ok();

    let user_manager = UserManager::new();
    let (redis_manager, pub_sub_rx) = RedisManager::new().await;

    let app_state = AppState::new(user_manager, redis_manager);
    let arc_data = Arc::new(Mutex::new(app_state));

    // run getting redis messages as an independent task
    let cloned_data = Arc::clone(&arc_data);
    tokio::spawn(RedisManager::broadcast_message_to_users(pub_sub_rx, cloned_data));

    let port = std::env::var("WSS_PORT").unwrap_or_else(|_|"8081".to_string());
    let address = format!("127.0.0.1:{}",port);

    println!("starting the wss server at : {}", port);

    let tcp_listener = TcpListener::bind(address).await.expect("Failed to bind to the given port");

    while let Ok((stream, socket_addr)) = tcp_listener.accept().await {
        let cloned_data = Arc::clone(&arc_data);
        // each client conn is a new task
        tokio::spawn(handle_connection(stream, socket_addr, cloned_data));
    }

}
