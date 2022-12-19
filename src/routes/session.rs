use crate::Sessions;
use crate::User;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_db_pools::deadpool_redis::redis::{AsyncCommands, RedisResult};
use rocket_dyn_templates::{context, Template};
// non of tracing macros is working in this module.
use tracing_attributes::instrument;
use uuid::Uuid;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

// TODO: Validation?
#[derive(FromForm, Debug)]
struct Login<'r> {
    username: &'r str,
    password: &'r str,
}

#[macro_export]
macro_rules! session_uri {
    ($($t:tt)*) => (rocket::uri!("/session", $crate::routes::session:: $($t)*))
}

// not particularly good name for macro, because macro with this name is already in rocket.
pub use session_uri as uri;

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(rocket::uri!("/"))
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    if let Some(flash) = flash {
        let kind = flash.kind();
        let message = flash.message();
        Template::render(
            "login",
            context! {
                kind,
                message
            },
        )
    } else {
        Template::render("login", context! {})
    }
}

// this request  as many other can end in error state.
#[post("/login", data = "<login>")]
async fn post_login(
    jar: &CookieJar<'_>,
    pool: &Sessions,
    login: Form<Login<'_>>,
) -> Result<Flash<Redirect>, Status> {
    // this will write to file, it works but too far off from proper logging.
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("./logs/info")
        .await
        .unwrap();
    file.write_all(b"async function printing info while login.")
        .await
        .unwrap();

    if login.username == "Sergio" && login.password == "password" {
        let id = Uuid::new_v4().to_string();
        let mut redis = match pool.get().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error while get redis connection from the pool. {}", e);
                return Err(Status::InternalServerError);
            }
        };

        // TODO: From / to vector of strings, it's  a bit ugly right now and not using type.
        let Ok(_unused) = redis
            .hset_multiple(
                &id,
                &[
                    ("id", id.clone()),
                    ("name", login.username.to_string()),
                    ("role", "user".to_string()),
                ],
            )
            .await as RedisResult<()> else
             {
                tracing::error!("Error while setting muliply values for User in Sessions.");
                return Err(Status::InternalServerError);
            };

        jar.add_private(Cookie::new("session_id", id));
        Ok(Flash::success(Redirect::to(rocket::uri!("/")), "OK"))
    } else {
        Ok(Flash::error(
            Redirect::to(uri!(login_page)),
            "Invalid username/password.",
        ))
    }
}

#[instrument]
#[get("/logout")]
async fn logout(jar: &CookieJar<'_>, pool: &Sessions) -> Result<Flash<Redirect>, Status> {
    let cookie = match jar.get_private("session_id") {
        Some(c) => c,
        None => return Ok(Flash::success(Redirect::to(rocket::uri!("/")), "OK")),
    };
    jar.remove_private(Cookie::named("session_id"));
    let mut redis = match pool.get().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Unable to get redis connection from pool. {}", e);
            return Err(Status::InternalServerError);
        }
    };
    let Ok(_unused) = redis.expire(cookie.value(), 1).await as RedisResult<()> else{
                tracing::error!("Error while expiring session for User.");
                return Err(Status::InternalServerError);
    };
    Ok(Flash::success(
        Redirect::to(rocket::uri!("/")),
        "Successfully logged out.",
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![/*no_auth_index,*/ login, login_page, post_login, logout]
}
