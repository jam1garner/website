#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
mod blog_data;

use rocket::response::NamedFile;
use rocket_contrib::templates::Template;

use std::path::{Path, PathBuf};

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
}])}

#[get("/<file..>")]
fn css(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("css/").join(file)).ok()
}

#[get("/<file>")]
fn blog_post(file: String) -> Option<Template> {
    println!("{}", file);
    Some(Template::render("post", blog_data::get_post_data(&file[..])?))
}

#[get("/raw/<file>")]
fn blog_post_raw(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("posts/").join(file)).ok()
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/css", routes![css])
        .mount("/blog/", routes![blog_post, blog_post_raw])
        .launch();
}
