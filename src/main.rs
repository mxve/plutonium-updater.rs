use colored::*;
use std::{fs, path::Path, str};
use clap::Parser;

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
    let cdn_get = easy_http_request::DefaultHttpRequest::get_from_url_str(
        "https://cdn.plutonium.pw/updater/prod/info.json",
    )
    .unwrap()
    .send()
    .unwrap();

    let mut downloader = downloader::Downloader::builder().build().unwrap();

    let resp: Info = serde_json::from_str(&str::from_utf8(&cdn_get.body).unwrap()).unwrap();

    for resp_file in resp.files {
        let file_path = Path::join(install_dir, Path::new(&resp_file.name));
        let file_dir = file_path.parent().unwrap();

        fs::create_dir_all(&file_dir).unwrap_or_else(|error| {
            panic!("{} {:?}", "Error:".bright_red(), error);
        });

        if file_path.exists() {
            let mut sha1 = sha1_smol::Sha1::new();
            sha1.update(&fs::read(&file_path).unwrap());
            let local_hash = sha1.digest().to_string();

            if &local_hash == &resp_file.hash {
                println!("{}: {}", "Checked".cyan(), resp_file.name);
                continue;
            } else {
                fs::remove_file(&file_path).unwrap_or_else(|error| {
                    panic!("{} {:?}", "Error:".bright_red(), error);
                });
            }
        }

        let url = format!("{}{}", &resp.base_url, &resp_file.hash);
        let dl = downloader::Download::new(&url).file_name(&file_path);

        let result = downloader.download(&[dl]).unwrap();
        for r in result {
            match r {
                Err(e) => println!("{} {}", "Error:".bright_red(), e.to_string()),
                Ok(_) => println!("{}: {}", "Downloaded".bright_yellow(), &resp_file.name),
            };
        }
    }
}

#[cfg(windows)]
fn setup_env() {
    // Enable color support
    colored::control::set_virtual_terminal(true);
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
