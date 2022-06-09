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

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Connecting to database...");
    database::connect().await;

    log::info!("Run migration...");
    utils::migration::migrate().await;

    let rocket = rocket::build();

    use fairings::*;

    routes::mount(rocket)
        .attach(cors::new())
        .attach(ratelimit::new())
        .attach(auth::new())
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", ratelimit::routes())
        .mount("/", auth::routes())
}
