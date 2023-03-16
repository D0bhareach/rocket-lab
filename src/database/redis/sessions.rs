use rocket::request::{self, FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::{deadpool_redis, Database};

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

// Sessons is wrapper around connection pool for redis.
#[derive(Database)]
#[database("redis_sessions")]
pub struct Sessions(deadpool_redis::Pool);

impl std::fmt::Debug for Sessions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Struct Sessions: Redis Connection from deadpool_redis::Pool.")
    }
}

// This will make Sessions request guard. It's possible to get Sessions from request.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Sessions {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Sessions, ()> {
        let pool = request.guard::<Sessions>().await.unwrap();
        let mut file = match File::open("./logs/info").await {
            Ok(f) => f,
            Err(_) => File::create("./logs/info").await.unwrap(),
        };
        file.write_all(b"async function printing info.")
            .await
            .unwrap();
        Outcome::Success(pool)
    }
}
