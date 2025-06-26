use actix_web::web::Data;
use r2d2_redis::{r2d2::{self, Pool}, RedisConnectionManager};

pub struct AppState{
    pub redis_pool: Pool<RedisConnectionManager>,
}

pub fn init_app_state() -> Data<AppState>{
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_e|String::from("redis://127.0.0.1:6379"));
    let manager = RedisConnectionManager::new(redis_url).expect("Failed to create redis manager");
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create Redis Pool");
    let state = Data::new(AppState {redis_pool:pool});
    state
}

// TODO: ORGANIZE THE ROUTES

#[macro_export]
macro_rules! init_app {
    ($state:expr) => {
        actix_web::App::new()
            .app_data($state.clone())
            .service(
                actix_web::web::scope("/api")
                .service(crate::handlers::health::hello_world)
                .service(crate::handlers::order::create::create_order)
                .service(crate::handlers::order::cancel::cancel_order)
            )
    };
}