use rocket::request::{self, FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::{deadpool_redis, Database};
use tracing::info;
use tracing_attributes::instrument;

#[derive(Database)]
#[database("redis_sessions")]
pub struct Sessions(deadpool_redis::Pool);

// impl Sessions {
//     pub async fn get_connection(&self) -> deadpool_redis::Connection {
//         self.0.get().await.unwrap();
//     }
// }
impl std::fmt::Debug for Sessions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Struct Sessions: Redis Connection from deadpool_redis::Pool.")
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Sessions {
    type Error = ();

    #[instrument]
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Sessions, ()> {
        // tracing::event!(tracing::Level::INFO, "get sessions from request.");
        let pool = request.guard::<Sessions>().await.unwrap();
        // tracing::span!(tracing::Level::INFO, "my_sessions_span")
        //     .in_scope(|| tracing::event!(tracing::Level::INFO, "MESSAGE FROM SESSIONS!!"));

        // let span = tracing::span!(tracing::Level::INFO, "my_span");
        // let _ = span.in_scope(|| tracing::info!("asdfsdffsdfsdfasdfsafas@#@$#$#$@$"));
        info!("async function printing info.");
        Outcome::Success(pool)
    }
}
