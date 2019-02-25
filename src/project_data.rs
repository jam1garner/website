use rocket_contrib::json::JsonValue;
use std::fs;

pub fn get_projects() -> Option<JsonValue> {
    Some(JsonValue(
        serde_json::from_str(
            &fs::read_to_string("templates/projects.json").ok()?[..]
        ).ok()?
    ))            
}
