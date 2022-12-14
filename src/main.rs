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
fn get_env() -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(2);
    for item in dotenvy::dotenv_iter().unwrap() {
        let (key, val) = item.unwrap();
        map.insert(key, val);
    }
    map
}

#[launch]
fn rocket() -> _ {
    let envs_map = get_env();
    let redis_url = get_connection_info(
        envs_map.get("redis_password").unwrap(),
        envs_map.get("redis_host").unwrap(),
        envs_map.get("redis_port").unwrap(),
    );

    // read configs form Rocket.toml
    let config = Config::figment();

    let figment = Figment::from(config)
        .merge((
            "databases.redis_sessions",
            rocket_db_pools::Config {
                url: redis_url,
                min_connections: Some(1),
                max_connections: 2usize,
                connect_timeout: 3u64,
                idle_timeout: Some(20),
            },
        ))
        .merge(("secret_key", envs_map.get("secret_key").unwrap()));

    let rocket = rocket::custom(figment);
    rocket
        .attach(Template::fairing())
        .attach(Sessions::init())
        .register("/", error_handler::handlers())
        .mount("/session", routes::session::routes())
        .mount("/", routes::index::routes())
        .mount("/public", FileServer::from(relative!("static")).rank(3))
}
