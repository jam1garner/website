#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate rss;
extern crate serde_json;
mod blog_data;
mod project_data;
mod feed;
mod compiler;

use rocket::response::{content, Redirect, Stream, NamedFile};
use rocket::data::Data;
use rocket_contrib::templates::Template;
use rocket_contrib::json::JsonValue;

use std::path::{Path, PathBuf};
use std::process::ChildStdout;

#[get("/")]
fn index() -> Template {
    Template::render("index", blog_data::get_posts())
}

#[get("/posts")]
fn posts() -> Option<JsonValue> {
    blog_data::get_posts()
}

#[get("/blog")]
fn blog() -> Redirect {
    Redirect::to("/")
}

#[get("/projects")]
fn projects() -> Option<Template> {
    Some(Template::render("projects", project_data::get_projects()?))
}

#[get("/<file..>")]
fn img(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("img/").join(file)).ok()
}

#[get("/<file..>")]
fn css(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("css/").join(file)).ok()
}

#[get("/<file..>")]
fn js(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("js/").join(file)).ok()
}

#[get("/unlisted/<file>")]
fn private_blog_post(file: String) -> Option<Template> {
    Some(Template::render("post", blog_data::get_post(&file[..], false)?))
}

#[get("/<file>")]
fn blog_post(file: String) -> Option<Template> {
    Some(Template::render("post", blog_data::get_post(&file[..], true)?))
}

#[get("/raw/<file>")]
fn blog_post_raw(file: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("posts/").join(file)).ok()
}

#[post("/compile", format="text/plain", data="<file>")]
fn compile(file: Data) -> Option<content::Json<Stream<ChildStdout>>> {
    compiler::compile(file)
}

#[get("/compiler")]
fn compiler_explorer() -> Option<Template> {
    Some(
        Template::render(
            "compiler_explorer",
            // Template data, passes in the default program
            compiler::get_default()
                .unwrap_or(
                    json![{
                        "program" : "void main(){\n}"
                    }]
                )
        )
    )
} 

fn main() {
     rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, posts, blog, projects, compile, compiler_explorer])
        .mount("/css/", routes![css])
        .mount("/js/", routes![js])
        .mount("/img/", routes![img])
        .mount("/blog/", routes![blog_post, blog_post_raw, private_blog_post])
        .mount("/rss", routes![feed::rss_feed])
        .launch();
}
