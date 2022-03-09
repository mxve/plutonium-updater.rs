use colored::*;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    str,
};

pub fn get_body(url: &str) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    http_req::request::get(url, &mut res).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });

    res
}

pub fn get_body_string(url: &str) -> String {
    String::from_utf8(get_body(&url)).unwrap()
}

pub fn download_file(url: &str, file_path: &PathBuf) {
    let body = get_body(&url);

    let mut f = fs::File::create(&file_path).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });
    f.write_all(&body).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });
}