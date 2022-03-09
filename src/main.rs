use colored::*;
use std::{fs, io::Write, path::Path, str};

mod args;
mod http;

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

fn get_cdn_info(cdn_url: String) -> CdnInfo {
    serde_json::from_str(&http::get_body_string(&cdn_url)).unwrap()
}

fn file_get_sha1(path: &Path) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(&path).unwrap());
    sha1.digest().to_string()
}

fn get_revision(path: &Path) -> u16 {
    fs::read_to_string(&path)
        .unwrap_or_else(|_| String::from("0"))
        .parse::<u16>()
        .unwrap_or(0)
}

fn set_revision(path: &Path, revision: &u16) {
    let mut revision_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
    revision_file
        .write_all(revision.to_string().as_bytes())
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
}

fn update(args: args::Args) {
    let install_dir = Path::new(&args.directory);
    let cdn_info: CdnInfo = get_cdn_info(args.cdn_url);

    let revision_file_path = Path::join(install_dir, "version.txt");
    let revision: u16 = get_revision(&revision_file_path);

    if !args.silent {
        println!(
            "Remote revision: {}",
            cdn_info.revision.to_string().bright_purple()
        );
    }

    // only update if remote version number is bigger than local
    // or if explicitly requested to update
    if (revision >= cdn_info.revision) && !args.force {
        if !args.silent {
            println!("Local revision: {}", revision.to_string().green())
        };
        return;
    }

    if !args.silent {
        println!("Local revision: {}", revision.to_string().yellow())
    };

    // keep track of processed files
    let mut stats = UpdateStats {
        checked: 0,
        downloaded: 0,
        skipped: 0,
    };

    // iterate cdn files
    for cdn_file in cdn_info.files {
        // skip launcher files if not explicitly asked for
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
            // if local sha1 is the same as remote we skip the file
            // otherwise delete local file
            if file_get_sha1(&file_path) == cdn_file.hash {
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
            // local directory doesnt exist, create it
            // no need to check for existing files
            fs::create_dir_all(&file_dir).unwrap_or_else(|error| {
                panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
            });
        }

        // download file from cdn, using base url + file hash to build file url
        http::download_file(
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

// Setup OS specific stuff
#[cfg(windows)]
fn setup_env(no_color: bool) {
    if no_color {
        colored::control::SHOULD_COLORIZE.set_override(false);
    } else {
        colored::control::set_virtual_terminal(true).unwrap_or_else(|error| {
            println!("{:#?}", error);
            colored::control::SHOULD_COLORIZE.set_override(false);
        });
    }
}

#[cfg(not(windows))]
fn setup_env(no_color: bool) {
    if no_color {
        colored::control::SHOULD_COLORIZE.set_override(false);
    }
}

fn main() {
    let args = args::get();

    setup_env(args.no_color);

    if args.check {
        let cdn_info = get_cdn_info(args.cdn_url);
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
