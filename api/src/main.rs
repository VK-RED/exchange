use actix_web::{HttpServer};
use dotenv::dotenv;

pub mod handlers;
pub mod errors;
pub mod entrypoint;
pub mod services;

#[actix_web::main]
async fn main() {
    dotenv().ok();

    let port = std::env::var("PORT").unwrap_or_else(|_e|String::from("8080"));
    let state = entrypoint::init_app_state();

    let address = format!("127.0.0.1:{}",port);
    println!("The server is running at the PORT : {}", port);

    HttpServer::new(
        move||{
            init_app!(state)
        }
    )
    .bind(address)
    .expect("Error while Binding to the Socket")
    .run()
    .await.expect("Error while starting the server")


}
