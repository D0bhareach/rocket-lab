// use crate::user::User;
use crate::Sessions;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::Config;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;

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

// #[get("/")]
// fn index(user: User) -> Template {
//     Template::render(
//         "index",
//         context! {
//             title: " - Index Page Loggedin",
//             name: "Viacheslav",
//             items: vec!["towel", "slippers", "toothbrush"],
//             logged: true,
//             user_id: user.0,
//         },
//     )
// }
/*
#[get("/")]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(login_page))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(rocket::uri!("/"))
}
*/

#[get("/login")]
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
) -> Result<Redirect, Flash<Redirect>> {
    if login.username == "Sergio" && login.password == "password" {
        let id = Uuid::new_v4().to_string();
        let mut redis = match pool.get().await {
            Ok(r) => r,
            Err(e) => {
                return Err(Flash::error(
                    Redirect::to(uri!(login_page)),
                    format!("{:?}", e),
                ));
            }
        };

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
        //

        jar.add_private(Cookie::new("session_id", id));
        Ok(Redirect::to(rocket::uri!("/")))
    } else {
        Err(Flash::error(
            Redirect::to(uri!(login_page)),
            "Invalid username/password.",
        ))
    }
}

#[post("/logout")]
async fn logout(jar: &CookieJar<'_>, pool: Sessions) -> Flash<Redirect> {
    let cookie = jar.get_private("session_id").unwrap();
    jar.remove_private(Cookie::named("session_id"));
    let mut redis = pool.get().await.unwrap();
    let _: () = redis.expire(cookie.value(), 1).await.unwrap();
    Flash::success(Redirect::to(rocket::uri!("/")), "Successfully logged out.")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![/*no_auth_index, login,*/ login_page, post_login, logout]
}
