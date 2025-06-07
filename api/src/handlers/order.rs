use actix_web::{post, HttpResponse, Responder};

#[post("/order")]
async fn create_order() -> impl Responder{
    
    // TODO: ADD USER_ID CHECK
    
    /*
        CREATE AN ORDER WITH ORDER ID 
        SUBSCRIBE TO THE PUB SUB WITH THE ORDERID
        PUSH THE ORDER TO REDIS QUEUE
    */

    // TODO: FIX THIS LATER
    HttpResponse::Ok().json("String Order Created Successfully !")
}