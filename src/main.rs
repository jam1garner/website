#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

use rocket_contrib::templates::Template;

#[get("/")]
fn index() -> Template {
    Template::render("index", 

json![{
    "posts" : [
        {
            "title": "test",
            "date": "01-01-2018"
        },
        {
            "title": "test 2",
            "date": "02-03-2019"
        }
    ],
    "test" : "3"
}]
)}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index]).launch();
}
