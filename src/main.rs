use clap::Parser;
use colored::*;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str,
};

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

fn get_cdn_info() -> CdnInfo {
    serde_json::from_str(&http_get_body_string(
        "https://cdn.plutonium.pw/updater/prod/info.json",
    ))
    .unwrap()
}

fn file_get_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(&path).unwrap());
    sha1.digest().to_string()
}

fn get_revision(path: &PathBuf) -> u16 {
    fs::read_to_string(&path)
        .unwrap_or(String::from("0"))
        .parse::<u16>()
        .unwrap_or(0)
}

fn set_revision(path: &PathBuf, revision: &u16) {
    let mut revision_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
    revision_file
        .write(revision.to_string().as_bytes())
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CdnInfo {
    revision: u16,
    base_url: String,
    files: Vec<CdnFile>,
}

#[derive(serde::Deserialize)]
struct CdnFile {
    name: String,
    // size: u32,
    hash: String,
}

#[derive(Debug)]
struct UpdateStats {
    checked: u8,
    downloaded: u8,
    skipped: u8,
}

fn update(args: Args) {
    let install_dir = Path::new(&args.directory);
    let cdn_info: CdnInfo = get_cdn_info();

    let revision_file_path = Path::join(&install_dir, "version.txt");
    let revision: u16 = get_revision(&revision_file_path);

    if !args.silent {
        println!(
            "Remote revision: {}",
            String::from(cdn_info.revision.to_string()).bright_purple()
        );
    }

    if (revision >= cdn_info.revision) && !args.force {
        if !args.silent {
            println!(
                "Local revision: {}",
                String::from(revision.to_string()).green()
            )
        };
        return;
    }

    if !args.silent {
        println!(
            "Local revision: {}",
            String::from(revision.to_string()).yellow()
        )
    };

    let mut stats = UpdateStats {
        checked: 0,
        downloaded: 0,
        skipped: 0,
    };

    // iterate cdn files
    for cdn_file in cdn_info.files {
        if cdn_file.name.starts_with("launcher") && !args.launcher {
            if !args.quiet && !args.silent {
                println!("{}: {}", "Skipped".bright_blue(), &cdn_file.name);
            };
            stats.skipped += 1;
            continue;
        }

        let file_path = Path::join(install_dir, Path::new(&cdn_file.name));
        let file_dir = file_path.parent().unwrap();

        if file_path.exists() {
            if &file_get_sha1(&file_path) == &cdn_file.hash {
                if !args.quiet && !args.silent {
                    println!("{}: {}", "Checked".cyan(), cdn_file.name)
                };
                stats.checked += 1;
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
        if !args.quiet && !args.silent {
            println!("{}: {}", "Downloaded".bright_yellow(), &cdn_file.name)
        };
        stats.downloaded += 1;
    }

    set_revision(&revision_file_path, &cdn_info.revision);

    if !args.silent {
        println!(
            "{} hashes checked, {} files downloaded, {} files skipped",
            stats.checked, stats.downloaded, stats.skipped
        );
    };
}

#[cfg(windows)]
fn setup_env() {
    // Enable color support
    colored::control::set_virtual_terminal(true).unwrap_or_else(|error| {
        println!("{:#?}", error);
        colored::control::SHOULD_COLORIZE.set_override(false);
    });
}

#[cfg(not(windows))]
fn setup_env() {
    // Empty for now
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Installation directory
    #[clap(short, long, default_value = "plutonium")]
    directory: String,

    /// Force file hash check, even if version matches
    #[clap(short, long)]
    force: bool,

    /// Download launcher assets
    #[clap(short, long)]
    launcher: bool,

    /// Hide file actions
    #[clap(short, long)]
    quiet: bool,

    /// Completely hide non-error output
    #[clap(short, long)]
    silent: bool,

    /// Check for update, returns exit code 0 for up to date and 1 for outdated
    #[clap(short, long)]
    check: bool,
}

fn main() {
    let args = Args::parse();

    setup_env();

    if args.check {
        let cdn_info = get_cdn_info();
        let revision = get_revision(&Path::join(Path::new(&args.directory), "version.txt"));

        if cdn_info.revision > revision {
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }
    update(args);

    std::process::exit(0);
}
