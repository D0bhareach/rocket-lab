#[macro_use]
extern crate rocket;
mod database;
mod error_handler;
mod routes;
mod user;
use crate::database::redis::sessions::Sessions;
use crate::user::user::User;
use dotenvy;
use rocket::fs::{relative, FileServer};
use rocket::Config;
use rocket_db_pools::figment::Figment;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use urlencoding::encode;

use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::subscribe::CollectExt;
use tracing_subscriber::{fmt, Registry};

// pages: one, two, three, dashboard.
// When logged must create session id.
// Then save session id to redis db.
// Simulate user loading from postgresql save name to session:id
// Need user struct. This stuct will need to serialize to redis db.
// Create other pages to simulate traveling to different urls with current session.
// Probably make some parts of menu not accessable or some url not accessabe to some users.
// Need to create user roles enum
// Create starting script to start all common tasks that I will need.
// Realize logout mechanism.
// Realize cookies timeout mechanism.
// Realize session timeout mechanism. Need update each time user interacts with server.
// Need structs for context of each page. Need separate module for them.
//
// TODO: Use simple unwrap code for now. Services required to be wired before server is
// started. Change error handling to logging error and panic!
fn get_connection_info(redis_password: &str, redis_host: &str, redis_port: &str) -> String {
    format!(
        "redis://:{}@{}:{}/",
        encode(redis_password),
        redis_host,
        redis_port
    )
}

// get corresponding  key / values from .env file
#[tracing::instrument]
fn get_env() -> HashMap<String, String> {
    tracing::info!("get_env method");
    let mut map = HashMap::with_capacity(2);
    for item in dotenvy::dotenv_iter().unwrap() {
        let (key, val) = item.unwrap();
        map.insert(key, val);
    }
    map
}

#[launch]
fn rocket() -> _ {
    // ******** SETTING TRACING ***********
    // TODO: path to file better be passed from config.
    // Instead of rolling will need to use rolling-file-rs, or my own implementation
    let info_file = rolling::daily("./logs", "info");
    let (info_out, _handle) = non_blocking(info_file);

    let debug_file = rolling::daily("./logs", "info");
    let (debug_out, _handle) = non_blocking(debug_file);

    let warn_file = rolling::daily("./logs", "warning");
    let (warn_out, _handle) = non_blocking(warn_file);

    let error_file = rolling::daily("./logs", "error");
    let (error_out, _handle) = non_blocking(error_file);

    let info_subscriber = fmt::Subscriber::default().with_writer(
        info_out
            .with_min_level(tracing::Level::INFO)
            .with_max_level(tracing::Level::INFO),
    );

    let debug_subscriber = fmt::Subscriber::default().pretty().with_writer(
        debug_out
            .with_min_level(tracing::Level::DEBUG)
            .with_max_level(tracing::Level::DEBUG),
    );
    let warning_subscriber = fmt::Subscriber::default()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(
            warn_out
                .with_min_level(tracing::Level::WARN)
                .with_max_level(tracing::Level::WARN),
        );

    let error_subscriber = fmt::Subscriber::default()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(
            error_out
                .with_min_level(tracing::Level::ERROR)
                .with_max_level(tracing::Level::ERROR),
        );

    let collector = Registry::default()
        .with(info_subscriber)
        .with(debug_subscriber)
        .with(warning_subscriber)
        .with(error_subscriber);

    tracing::collect::set_global_default(collector).expect("Tracing must be ready for the App.");

    tracing::span!(tracing::Level::INFO, "my_span")
        .in_scope(|| tracing::event!(tracing::Level::INFO, "BEFORE READING ENV FILE"));

    //
    let envs_map = get_env();
    let redis_url = get_connection_info(
        envs_map.get("redis_password").unwrap(),
        envs_map.get("redis_host").unwrap(),
        envs_map.get("redis_port").unwrap(),
    );
    tracing::info!("After reading .env file. Port: {}", envs_map["redis_port"]);

    // read configs form Rocket.toml
    let config = Config::figment();

    let figment = Figment::from(config)
        .join((
            "databases.redis_sessions",
            rocket_db_pools::Config {
                url: redis_url,
                // not sure if it taken from Rocket.toml
                min_connections: None,
                max_connections: 0,
                connect_timeout: 0,
                idle_timeout: None,
            },
        ))
        .merge(("secret_key", envs_map.get("secret_key").unwrap()));

    let rocket = rocket::custom(figment);
    //  *********** BUILDING AND STARTING ROCKET ************
    rocket
        .attach(Template::fairing())
        .attach(Sessions::init())
        .register("/", error_handler::handlers())
        .mount("/session", routes::session::routes())
        .mount("/", routes::index::routes())
        .mount("/public", FileServer::from(relative!("static")).rank(3))
}
