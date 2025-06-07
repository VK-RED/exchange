use actix_web::{Responder, get};

#[get("/")]
async fn hello_world() -> impl Responder{
    "hello_world"
}