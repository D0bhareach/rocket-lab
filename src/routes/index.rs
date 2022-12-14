use crate::User;
use rocket_dyn_templates::tera::Context;
use rocket_dyn_templates::Template;

#[get("/")]
fn index(user: Option<User>) -> Template {
    let mut context = Context::new();
    context.insert("title", " - Index Page");
    match user {
        Some(u) => {
            context.insert("name", &u.name);
            context.insert("items", &vec!["Rocket", "docs", "repository"]);
            context.insert("logged", &true);
        }
        None => {
            context.insert("name", "Guest");
            context.insert("logged", &false);
        }
    }

    Template::render("index", context.into_json())
}
pub fn routes() -> Vec<rocket::Route> {
    routes![index,]
}
