use rocket::request::{self, FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::{deadpool_redis, Database};

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
