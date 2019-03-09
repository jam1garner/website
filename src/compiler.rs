use std::fs;
use std::process::{Command, Stdio, ChildStdout};

use rocket::data::Data;
use rocket::response::{content, Stream};
use rocket_contrib::json::JsonValue;

pub fn compile(file: Data) -> Option<content::Json<Stream<ChildStdout>>> {
    let process = Command::new("python3")
                    .arg("msclang_explorer/msclang.py")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .ok()?;
    
    file.stream_to(&mut process.stdin?).ok()?;

    Some(content::Json(Stream::from(process.stdout?)))
}

pub fn get_default() -> Option<JsonValue> {
    Some(
        json![{
            "program" : 
                fs::read_to_string("templates/default.c")
                    .ok()?
                    .split("\n")
                    .map(|x| format!("<div>{}</div>", x).to_string())
                    .collect::<String>()
        }]
    )
}
