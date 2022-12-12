use crate::Sessions;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
/*
 * All users except admin. must have fields relevant to user.
 */
// #[derive(Debug)]
pub struct User {
    // user id in db, as a second thought it's not secure must get info from db
    // when need it by session id.
    pub id: String,
    pub name: String,
    pub role: String,
    // other fields relevant to user cache.
}

// impl User {
//     pub fn new() -> Self {
//         User {}
//     }
// }
impl From<Vec<String>> for User {
    fn from(v: Vec<String>) -> Self {
        User {
            id: String::from(&v[1]),
            name: String::from(&v[3]),
            role: String::from(&v[5]),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, ()> {
        let pool = request
            .guard::<&'r Sessions>()
            .await
            .expect("Error get Connection.");
        let mut redis = pool.get().await.unwrap();
        let Some(cookie) = request.cookies().get_private("session_id") else{
            return Outcome::Forward(());
        };
        let s_id = cookie.value();
        let key = format!("session:{}", s_id);
        let arr: Vec<String> = redis.hgetall(key).await.unwrap();
        Outcome::Success(User::from(arr))
    }
}
