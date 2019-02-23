use rocket_contrib::json::JsonValue;
use comrak::{markdown_to_html, ComrakOptions};
use std::path::{Path};
use std::fs;
use regex::Regex;

fn post_to_html<P: AsRef<Path>>(path: P) -> String {
    let contents = fs::read_to_string(path)
        .expect("Unable to read from file");
    let mut options = ComrakOptions::default();
    options.unsafe_ = true;
    markdown_to_html(&contents[..], &options)
}

fn extract_first_image(markdown: &str) -> Option<String> {
    lazy_static! {
        static ref image_regex: Regex = Regex::new(r"!\[\]\(([^\^\n)]+)\)").unwrap();
    }
    // Return $0 from ![]($0), returns None if not found
    Some(
        image_regex.captures(markdown)?
        .get(1)?
        .as_str()
        .to_string()
    )
}

fn extract_post_title(markdown: &str) -> Option<String> {
    lazy_static! {
        static ref title_regex: Regex = Regex::new(r"(?m)^ *#(.+)").unwrap();
    }
    // Return the first #-level title, returns None if not found
    Some(
        title_regex.captures(markdown)?
        .get(1)?
        .as_str()
        .to_string()
    )
}

fn extract_post_timestamp(markdown: &str) -> Option<u64> {
    lazy_static! {
        static ref title_regex: Regex = Regex::new(r"<!-- *timestamp: *(\d+) *-->").unwrap();
    }
    // Return the first num from html comment in format timestamp:[num], returns None if not found
    Some(
        title_regex.captures(markdown)?
        .get(1)?
        .as_str()
        .parse::<u64>()
        .ok()?
    )
}

fn post_to_simple_json<P: AsRef<Path>>(path: P) -> Option<JsonValue> {
    let contents = fs::read_to_string(path)
        .expect("Unable to read from file");
    let image_url = extract_first_image(&contents[..]).unwrap_or_default();
    let title = extract_post_title(&contents[..])?;
    let timestamp = extract_post_timestamp(&contents[..]);
    Some(json![{}])
}

pub fn get_post(post_name: &str) -> Option<JsonValue> {
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

pub fn get_posts() -> Option<JsonValue> {
    Some(json![
        fs::read_dir("posts").ok()?
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|path| path.is_file() && path.extension().unwrap_or_default() == ".md")
            .filter_map(|path| post_to_simple_json(path))
            .collect::<Vec<JsonValue>>() 
    ])
}

