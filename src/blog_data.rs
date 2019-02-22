use rocket_contrib::json::JsonValue;
use comrak::{markdown_to_html, ComrakOptions};
use std::path::{Path};
use std::fs;

fn post_to_html<P: AsRef<Path>>(path: P) -> String {
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    markdown_to_html(&contents[..], &ComrakOptions::default())
}

pub fn get_post_data(post_name: &str) -> Option<JsonValue> {
    let post_path = format!("posts/{}.md", post_name);
    let path = Path::new(&post_path[..]);
    if path.is_file() {
        Some(json![{
            "post" : {
                "name": "test",
                "date": 0,
                "body": post_to_html(path)
            }
        }])
    }
    else {
        None
    }
}
