#![feature(decl_macro)]

#[macro_use]
extern crate rocket;

use api::FeederEvent;
use internal::Storage;
use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::broadcast::channel;
use rocket::Config;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;

mod api;
pub mod internal;

#[launch]
fn rocket() -> _ {
    let figment = Config::figment().merge((
        "databases.storage",
        rocket_db_pools::Config {
            url: "mongodb://localhost:27017".into(),
            min_connections: Some(1),
            max_connections: 2,
            connect_timeout: 5,
            idle_timeout: None,
        },
    ));
    let config = Config {
        port: 3456,
        ..Config::default()
    };

    let allowed_origins = AllowedOrigins::all();

    let cors = CorsOptions {
        allowed_origins,
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: false,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .configure(config)
        .configure(figment)
        .attach(cors)
        .attach(Storage::init())
        .manage(channel::<FeederEvent>(10).0)
        .mount("/api/v1", api::routes())
        .mount("/", FileServer::from(relative!("static")))
}
