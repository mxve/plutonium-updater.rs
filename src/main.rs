use clap::Parser;
use colored::*;
use std::{fs, io::Write, path::Path, str};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    // revision: u16,
    base_url: String,
    files: Vec<PlutoFile>,
}

#[derive(serde::Deserialize)]
struct PlutoFile {
    name: String,
    // size: u32,
    hash: String,
}

fn update(directory: String) {
    let install_dir = Path::new(&directory);

    // get cdn info
    let mut body = Vec::new();
    http_req::request::get("https://cdn.plutonium.pw/updater/prod/info.json", &mut body)
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
    let resp: Info = serde_json::from_str(&str::from_utf8(&body).unwrap()).unwrap();

    // iterate cdn files
    for resp_file in resp.files {
        let file_path = Path::join(install_dir, Path::new(&resp_file.name));
        let file_dir = file_path.parent().unwrap();

        if file_path.exists() {
            let mut sha1 = sha1_smol::Sha1::new();
            sha1.update(&fs::read(&file_path).unwrap());
            let local_hash = sha1.digest().to_string();

            if &local_hash == &resp_file.hash {
                println!("{}: {}", "Checked".cyan(), resp_file.name);
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

        // download file
        let url = format!("{}{}", &resp.base_url, &resp_file.hash);
        let mut body = Vec::new();
        http_req::request::get(&url, &mut body).unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });

        // write file
        let mut f = fs::File::create(&file_path).unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
        f.write_all(&body).unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });

        println!("{}: {}", "Downloaded".bright_yellow(), &resp_file.name);
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
