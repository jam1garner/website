use rocket_contrib::json::JsonValue;
use comrak::{markdown_to_html, ComrakOptions};
use chrono::prelude::*;
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
        static ref IMAGE_REGEX: Regex = Regex::new(r"!\[\]\(([^\^\n)]+)\)").unwrap();
    }
    // Return $0 from ![]($0), returns None if not found
    Some(
        IMAGE_REGEX.captures(markdown)?
        .get(1)?
        .as_str()
        .to_string()
    )
}

fn extract_post_title(markdown: &str) -> Option<String> {
    lazy_static! {
        static ref TITLE_REGEX: Regex = Regex::new(r"(?m)^ *#(.+)").unwrap();
    }
    // Return the first #-level title, returns None if not found
    Some(
        TITLE_REGEX.captures(markdown)?
        .get(1)?
        .as_str()
        .to_string()
    )
}

fn extract_post_timestamp(markdown: &str) -> Option<i64> {
    lazy_static! {
        static ref TIMESTAMP_REGEX: Regex = Regex::new(r"<!-- *timestamp: *(\d+) *-->").unwrap();
    }
    // Return the first num from html comment in format timestamp:[num], returns None if not found
    Some(
        TIMESTAMP_REGEX.captures(markdown)?
        .get(1)?
        .as_str()
        .parse::<i64>()
        .ok()?
    )
}

fn post_to_simple_json<P: AsRef<Path>>(path: P) -> Option<JsonValue> {
    let contents = fs::read_to_string(path)
        .expect("Unable to read from file");
    let image_url = extract_first_image(&contents[..]).unwrap_or_default();
    let title = extract_post_title(&contents[..])?;
    let timestamp = extract_post_timestamp(&contents[..]).unwrap_or_default();

    Some(json![{
        "title": title,
        "thumbnail": image_url,
        "date": Utc.timestamp(timestamp, 0).format("%d %B %Y").to_string(),
        "timestamp": timestamp
    }])
}

pub fn get_post(post_name: &str) -> Option<JsonValue> {
    let post_path = format!("posts/{}.md", post_name);
    let path = Path::new(&post_path[..]);
    if path.is_file() {
        let post_data = post_to_simple_json(path)?;
        Some(json![{
             "title": post_data.get("title"),
             "thumbnail": post_data.get("thumbnail"),
             "date": post_data.get("date"),
             "timestamp": post_data.get("timestamp"),
             "body": post_to_html(path)
        })
    }
    else {
        None
    }
}

pub fn get_posts() -> Option<JsonValue> {
    let mut posts = 
            fs::read_dir("posts").ok()?
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "md")
            .filter_map(|path| post_to_simple_json(path))
            .collect::<Vec<JsonValue>>();

    posts.sort_by_key(|j| j["timestamp"].as_i64());
    posts.reverse();
    Some(json![{
        "posts": posts
    }])
}

