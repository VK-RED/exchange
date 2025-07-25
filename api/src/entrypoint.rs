use actix_web::web::Data;
use r2d2_redis::{r2d2::{self, Pool}, RedisConnectionManager};
use sqlx::{postgres::PgPoolOptions, Postgres};

pub struct AppState{
    pub redis_pool: Pool<RedisConnectionManager>,
    pub db_pool: sqlx::Pool<Postgres>,
}

pub async fn init_app_state() -> Data<AppState>{
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_e|String::from("redis://127.0.0.1:6379"));
    let manager = RedisConnectionManager::new(redis_url).expect("Failed to create redis manager");
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create Redis Pool");

    let default_db_url = String::from("postgres://postgres:postgres@localhost:5432/postgres");
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_|default_db_url);

    let db_pool = PgPoolOptions::new().connect(&database_url).await.expect("Failed to connect to DB!");
    
    let state = Data::new(AppState {redis_pool:pool, db_pool});
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
                .service(crate::handlers::order::cancel_all::cancel_all_orders)
                .service(crate::handlers::order::open_orders::get_all_open_orders)
                .service(crate::handlers::depth::get_depth)
                .service(crate::handlers::user::balance::get_user_balance)
                .service(crate::handlers::trade::get_trade_history)
            )
    };
}