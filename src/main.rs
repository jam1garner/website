#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate rss;
mod blog_data;
mod feed;

use rocket::response::NamedFile;
use rocket_contrib::templates::Template;
use rocket_contrib::json::JsonValue;

use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> Template {
    Template::render("index", blog_data::get_posts())
}

#[get("/posts")]
fn posts() -> Option<JsonValue> {
    blog_data::get_posts()
}

#[get("/<file..>")]
fn css(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("css/").join(file)).ok()
}

#[get("/<file>")]
fn blog_post(file: String) -> Option<Template> {
    Some(Template::render("post", blog_data::get_post(&file[..])?))
}

#[get("/raw/<file>")]
fn blog_post_raw(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("posts/").join(file)).ok()
}


fn main() {
     rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, posts])
        .mount("/css", routes![css])
        .mount("/blog/", routes![blog_post, blog_post_raw])
        .mount("/rss", routes![feed::rss_feed])
        .launch();
}
