#[macro_use]
extern crate rocket;
use rocket::config::SecretKey;
use rocket::fs::{relative, FileServer};
use rocket::request::{self, FromRequest, Outcome};
use rocket::{Config, Request};
use std::collections::HashMap;

use rocket_dyn_templates::{context, Template};
mod error_handler;
mod routes;
pub mod user;
// use error_handler;
//use deadpool_redis::Connection;
use dotenvy;
use rocket_db_pools::figment::Figment;
use rocket_db_pools::{deadpool_redis, Database};
use urlencoding::encode;
use user::User;

#[derive(Database)]
#[database("redis_sessions")]
pub struct Sessions(deadpool_redis::Pool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Sessions {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Sessions, ()> {
        let pool = request.guard::<Sessions>().await.unwrap();
        Outcome::Success(pool)
    }
}

#[get("/")]
async fn index(/*_user: User *//*, mut redis: Connection<Sessions>*/
) -> Template {
    // let _: () = redis.set("test", 4i32).await.unwrap();
    Template::render(
        "index",
        context! {
        title: " - Index Page",
        name: "Viacheslav",
        items: vec!["towel", "slippers", "toothbrush"],
        logged: false,
        },
    )
}
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
fn get_connection_info(redis_password: &str, redis_host: &str, redis_port: &str) -> String {
    format!(
        "redis://:{}@{}:{}/",
        encode(redis_password),
        redis_host,
        redis_port
    )
}

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
    let redis_url =
        get_connection_info(envs_map.get("redis_password").unwrap(), "127.0.0.1", "6379");

    // let config = Config {
    //     port: 8001,
    //     ..Config::debug_default()
    // };
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
        .merge(("log_level", "normal"))
        .merge(("secret_key", envs_map.get("secret_key").unwrap()));

    let rocket = rocket::custom(figment);
    rocket
        .attach(Template::fairing())
        .attach(Sessions::init())
        .register("/", error_handler::handlers())
        .mount("/session", routes::session::routes())
        .mount("/", routes![index])
        .mount("/public", FileServer::from(relative!("static")).rank(30))
}
