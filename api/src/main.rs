use actix_web::{App, HttpServer};
use dotenv::dotenv;

pub mod handlers;
pub mod errors;
#[actix_web::main]
async fn main() {
    dotenv().ok();

    let port = std::env::var("PORT").unwrap_or_else(|_e|String::from("8080"));
    let address = format!("127.0.0.1:{}",port);

    println!("The server is running at the PORT : {}", port);

    HttpServer::new(
        ||{
            App::new()
            .service(handlers::health::hello_world)
        }
    )
    .bind(address)
    .expect("Error while Binding to the Socket")
    .run()
    .await.expect("Error while starting the server")


}
