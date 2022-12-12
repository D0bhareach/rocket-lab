use rocket::{self, catch};
use rocket_dyn_templates::{context, Template};

#[catch(404)]
fn not_found() -> Template {
    Template::render("404", context! {})
    // "Static string."
}
#[catch(500)]
fn internal_error() -> Template {
    Template::render("500", context! {})
    // "Static string."
}

pub fn handlers() -> Vec<rocket::Catcher> {
    catchers![not_found, internal_error]
}
