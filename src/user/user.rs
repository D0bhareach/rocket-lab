use crate::Sessions;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_db_pools::deadpool_redis::redis::{
    AsyncCommands, ErrorKind, FromRedisValue, RedisResult, Value,
};
/*
 * All users except admin. must have fields relevant to user.
 */
#[derive(Debug, Default)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: String,
}

// TODO: User is a session's guard I will have two different types of user in the app.
// Id field in user is redundant, name can be safely empty string, as to role - because
// role is important I would rather refrain from creation of user if it not correspond to
// any role and each user-role does require separate type of user. It will make boiler plate
// codes or need for macros but will be more secure. The idea is to keep current data for user
// in the session and this data must be easily to emmited. Cache means not important data.
impl From<&Vec<Value>> for User {
    fn from(v: &Vec<Value>) -> Self {
        let mut user = User::default();
        if v.len() != 6 {
            return user;
        } else {
            for (idx, value) in v.iter().enumerate() {
                match value {
                    Value::Data(u) if idx == 1 => {
                        user.id = String::from_utf8(u.clone()).unwrap_or(String::new())
                    }
                    Value::Data(u) if idx == 3 => {
                        user.name = String::from_utf8(u.clone()).unwrap_or(String::new())
                    }
                    Value::Data(u) if idx == 5 => {
                        user.role = String::from_utf8(u.clone()).unwrap_or(String::new())
                    }
                    _ => continue,
                }
            }
        }
        user
    }
}

impl FromRedisValue for User {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {
            Value::Bulk(val) => {
                if val.len() < 6 {
                    Err((ErrorKind::TypeError, "").into())
                } else {
                    let user = User::from(val);
                    return Ok(user);
                }
            }
            _ => return Err((ErrorKind::TypeError, "").into()),
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
        let Some(cookie) = request.cookies().get_private("session_id") else {
            return Outcome::Forward(());
        };
        let s_id = cookie.value();
        let res: rocket_db_pools::deadpool_redis::redis::RedisResult<User> =
            redis.hgetall(s_id).await;
        match res {
            Ok(arr) => Outcome::Success(User::from(arr)),
            Err(e) => Outcome::Forward(()),
        }
    }
}
