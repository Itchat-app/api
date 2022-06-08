#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate lazy_static;

pub mod config;
pub mod database;
pub mod fairings;
pub mod guards;
pub mod routes;
pub mod structures;
pub mod utils;

use fairings::*;

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Connecting to database...");
    database::connect().await;

    log::info!("Run migration...");
    utils::migration::migrate().await;

    let auth = fairings::auth::Auth {
        ignore: vec![
            "/".into(),
            "/auth/accounts/register".into(),
            "/auth/accounts/verify".into(),
            "/auth/sessions/login".into(),
            "/ratelimit",
        ],
    };

    let rocket = rocket::build();

    let _ = routes::mount(rocket)
        .attach(ratelimit::RateLimiter)
        .attach(auth)
        .mount("/", ratelimit::routes())
        .mount("/", auth::routes())
        .launch()
        .await;
}
