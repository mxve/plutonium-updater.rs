use clap::Parser;
use colored::*;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str,
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CdnInfo {
    // revision: u16,
    base_url: String,
    files: Vec<CdnFile>,
}

#[derive(serde::Deserialize)]
struct CdnFile {
    name: String,
    // size: u32,
    hash: String,
}

fn http_get_body(url: &str) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    http_req::request::get(url, &mut res).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });

    res
}

fn http_get_body_string(url: &str) -> String {
    String::from_utf8(http_get_body(&url)).unwrap()
}

fn http_download(url: &str, file_path: &PathBuf) {
    let body = http_get_body(&url);

    let mut f = fs::File::create(&file_path).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });
    f.write_all(&body).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
    });
}

fn file_get_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(&path).unwrap());
    sha1.digest().to_string()
}

fn update(directory: String) {
    let install_dir = Path::new(&directory);
    let cdn_info: CdnInfo = serde_json::from_str(&http_get_body_string(
        "https://cdn.plutonium.pw/updater/prod/info.json",
    ))
    .unwrap();

    // iterate cdn files
    for cdn_file in cdn_info.files {
        let file_path = Path::join(install_dir, Path::new(&cdn_file.name));
        let file_dir = file_path.parent().unwrap();

        if file_path.exists() {
            if &file_get_sha1(&file_path) == &cdn_file.hash {
                println!("{}: {}", "Checked".cyan(), cdn_file.name);
                continue;
            } else {
                fs::remove_file(&file_path).unwrap_or_else(|error| {
                    panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
                });
            }
        } else {
            fs::create_dir_all(&file_dir).unwrap_or_else(|error| {
                panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
            });
        }

        http_download(
            &format!("{}{}", &cdn_info.base_url, &cdn_file.hash),
            &file_path,
        );
        println!("{}: {}", "Downloaded".bright_yellow(), &cdn_file.name);
    }
}

#[cfg(windows)]
fn setup_env() {
    // Enable color support
    colored::control::set_virtual_terminal(true).unwrap_or_else(|error| {
        panic!("\n\nError:\n{:?}", error);
    });
}

#[cfg(not(windows))]
fn setup_env() {
    // Empty for now
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "plutonium")]
    directory: String,
}

fn main() {
    let args = Args::parse();

    setup_env();
    update(args.directory);
}
