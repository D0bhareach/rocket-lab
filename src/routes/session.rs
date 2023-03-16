use crate::Sessions;
use crate::User;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;

// TODO: Validation?
#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str,
}

#[macro_export]
macro_rules! session_uri {
    ($($t:tt)*) => (rocket::uri!("/session", $crate::routes::session:: $($t)*))
}

pub use session_uri as uri;

/*
#[get("/")]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(login_page))
}
*/

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
    if login.username == "Sergio" && login.password == "password" {
        let id = Uuid::new_v4().to_string();
        let mut redis = match pool.get().await {
            Ok(r) => r,
            Err(_e) => return Err(Status::InternalServerError),
        };

        // TODO: From / to vector of strings
        let _: () = redis
            .hset_multiple(
                &id,
                &[
                    ("id", id.clone()),
                    ("name", login.username.to_string()),
                    ("role", "user".to_string()),
                ],
            )
            .await
            .unwrap();

        jar.add_private(Cookie::new("session_id", id));
        Ok(Flash::success(Redirect::to(rocket::uri!("/")), "OK"))
    } else {
        Ok(Flash::error(
            Redirect::to(uri!(login_page)),
            "Invalid username/password.",
        ))
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>, pool: &Sessions) -> Result<Flash<Redirect>, Status> {
    let cookie = match jar.get_private("session_id") {
        Some(c) => c,
        None => return Ok(Flash::success(Redirect::to(rocket::uri!("/")), "OK")),
    };
    jar.remove_private(Cookie::named("session_id"));
    let mut redis = match pool.get().await {
        Ok(r) => r,
        Err(_e) => return Err(Status::InternalServerError),
    };
    let _: () = redis.expire(cookie.value(), 1).await.unwrap();
    Ok(Flash::success(
        Redirect::to(rocket::uri!("/")),
        "Successfully logged out.",
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![/*no_auth_index,*/ login, login_page, post_login, logout]
}
