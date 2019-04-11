use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use rocket::Data;
use rocket::response::{Content, Stream};
use rocket_contrib::templates::Template;
use rocket::http::ContentType;

use nus3audio::{Nus3audioFile, AudioFile};

fn clean_tmp() -> bool {
    std::process::Command::new("find")
        .args(&["/tmp/nus3audio", "-type", "f", "-amin", "+10", "-delete"])
        .output()
        .is_ok()
}

fn vgaudio_convert<S: AsRef<str>>(in_file: S, out_lopus: S) -> Option<std::process::Output> {
    let args = ["vgaudio/VGAudioCli.dll", "-c", in_file.as_ref(), out_lopus.as_ref(),
                "--bitrate", "64000", "--CBR", "--opusheader", "namco"];
    std::process::Command::new("dotnet")
        .args(&args)
        .output()
        .ok()
}

#[get("/")]
pub fn nus3audio_converter() -> Option<Template> {
    Some(
        Template::render(
            "nus3audio_converter", json![{}]
        )
    )
}

#[post("/upload?<name>", data = "<file>")]
pub fn nus3audio_upload(name: String, file: Data) -> Option<String> {
    clean_tmp();
    let id = format!("{:032x}", rand::random::<u128>());
    let ext = Path::new(&name).extension()?.to_str()?;
    let name = Path::new(&name).file_stem()?.to_str()?;
    let path = format!("/tmp/nus3audio/{id}/{name}", id = id,
                       name = name)
                       .to_string();
    let path_with_ext = format!("{}.{}", path, ext);

    // Attempt to convert given file to wav
    fs::create_dir_all(format!("/tmp/nus3audio/{}", id)).ok()?;
    file.stream_to_file(&path_with_ext).ok()?;
    let lopus_path = path.clone() + ".lopus";
    if let Some(output) = vgaudio_convert(&path_with_ext, &lopus_path) {
        println!("{}", std::str::from_utf8(&output.stdout[..]).ok()?);
    }
    
    let mut nus3_file = Nus3audioFile::new();
    let mut lopus_data = vec![];
    
    // Build nus3audio file from lopus
    fs::File::open(&lopus_path).ok()?
        .read_to_end(&mut lopus_data).ok()?;
    nus3_file.files.push(
        AudioFile {
            name: String::from(name),
            data: lopus_data,
            id: 0,
        }
    );
    let mut file_bytes = Vec::with_capacity(nus3_file.calc_size());
    nus3_file.write(&mut file_bytes);
    let nus3audio_path = path.clone() + ".nus3audio";
    fs::File::create(&nus3audio_path).ok()?
        .write_all(&file_bytes[..]).ok()?;
    
    // Cleanup unneeded files
    fs::remove_file(lopus_path).ok();
    fs::remove_file(&path_with_ext).ok();
    Some(
        format!("/nus3audio/files/{id}/{name}.nus3audio", id=id, name=name)
    )
}

#[get("/files/<id>/<name>")]
pub fn nus3audio_download(id: String, name: String) -> Option<Content<Stream<File>>> {
    Some(Content(ContentType::Binary, Stream::from(
        File::open(
            format!("/tmp/nus3audio/{id}/{name}", id = id, name = name)
        ).ok()?
    )))
}
