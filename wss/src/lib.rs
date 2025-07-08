use std::{net::SocketAddr, sync::{Arc}};
use futures_util::{pin_mut, SinkExt, StreamExt};
use serde::Deserialize;
use tokio::{net::TcpStream, sync::{mpsc, Mutex}};
pub use redis::RedisManager;
use tokio_tungstenite::tungstenite::Message;
pub use user::UserManager;

mod redis;
mod user;

pub struct AppState {
    pub user_manager : UserManager,
    pub redis: RedisManager,
}

impl AppState {
    pub fn new(user_manager:UserManager, redis:RedisManager) -> Self{
        Self { user_manager, redis }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientMessage {
    pub message_type:String,
    pub channel: String, 
}

/// Client's inbound and outbound
/// messages are handles here.
pub async fn handle_connection(
    stream: TcpStream,
    socket_addr: SocketAddr,
    app_state: Arc<Mutex<AppState>>
){

    // convert tcp stream to ws stream

    let try_stream = tokio_tungstenite::accept_async(stream).await;

    if let Err(e) = try_stream {
        println!("failed handshake : {}", e);
        return;
    }

    let ws_stream = try_stream.unwrap();

    let (mut sink, mut stream) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();    

    let mut guard = app_state.lock().await;
    let user_id = socket_addr.port().to_string();

    println!("user connected : {}", user_id);

    guard.user_manager.register_user(user_id.clone(), tx.clone());

    // drop the guard
    drop(guard);

    let read_fut = async {

        while let Some(try_message) = stream.next().await {

            match try_message {
                Ok(message) => {                    

                    let str_message = message.to_string();                
                    let mut guard = app_state.lock().await;
                    let try_deserialized: Result<ClientMessage, serde_json::Error> = serde_json::from_str(&str_message);

                    match try_deserialized{
                        Ok(client_message) => {
                
                            if client_message.message_type == "SUBSCRIBE" {

                                let channel = client_message.channel;

                                let user_manager = &mut guard.user_manager;
                                user_manager.subscribe(user_id.clone(), channel.clone()).await;

                                let redis_manager = &mut guard.redis;
                                let res = redis_manager.subscribe(&channel, user_id.clone()).await;

                                if let Err(e) = res {
                                    println!("err : {} while subscribing for user: {} in channel : {}", e, user_id, channel);
                                }
                            }
                            else if client_message.message_type == "UNSUBSCRIBE" {
                               
                                let channel = client_message.channel;

                                let user_manager = &mut guard.user_manager;
                                user_manager.unsubscribe(user_id.clone(), channel.clone()).await;

                                let redis_manager = &mut guard.redis;
                                let res = redis_manager.unsubcribe(&channel, &user_id).await;

                                if let Err(e) = res {
                                    println!("err : {} while unsubscribing for user: {} in channel : {}", e, user_id, channel);
                                }
                            }
                            else{
                                println!("unhandled message type : {:?}", client_message);

                                let message = format!("{} is unhandled", str_message);
                                if let Err(e) = tx.send(message) {
                                    println!("error : {} while sending message to rx", e);
                                }
                            }
                        },
                        Err(e) => {
                            println!("error: {}, while deserializing", e);

                            let message = String::from("Invalid Message !");
                            if let Err(e) = tx.send(message) {
                                println!("error : {} while sending message to rx", e);
                            }
                        }
                    }

                },
                Err(e) => {
                    println!("error : {} while receiving message from client", e);
                }
            }
        }
    };

    let send_fut = async {
        while let Some(msg) = rx.recv().await {

            let message = Message::text(msg);
            let send_res = sink.send(message).await;

            if let Err(e) = send_res {
                println!("error : {} while sending res to client !",e)
            }
        }
    };

    pin_mut!(read_fut, send_fut);
    
    futures_util::future::select(read_fut, send_fut).await;

    println!("cleaning up user : {} !!", user_id);

    let mut guard = app_state.lock().await;
    
    let subscribed_channels = guard.user_manager.remove_user(user_id.clone()).await;

    if let Some(channels) = subscribed_channels {
        guard.redis.unsubscribe_channels(&channels, &user_id).await;
    }



}
