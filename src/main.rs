use colored::*;
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
    str,
};

mod args;
mod http;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CdnInfo {
    product: String,
    revision: u16,
    base_url: String,
    files: Vec<CdnFile>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct CdnFile {
    name: String,
    size: u32,
    hash: String,
}

#[derive(Debug)]
struct UpdateStats {
    checked: u8,
    downloaded: u8,
    skipped: u8,
}

fn parse_info(info: &str) -> CdnInfo {
    serde_json::from_str(info).unwrap()
}

// Read file to serde json CdnInfo
fn read_info_file(filepath: &Path) -> CdnInfo {
    let info_file = fs::read_to_string(&filepath).unwrap_or_else(|_| {
        "{\"product\":\"plutonium-core-unknown\",
                 \"revision\":0,
                 \"baseUrl\":\"\",
                 \"files\":[]}"
            .to_string()
    });
    parse_info(&info_file)
}

// Write serde json CdnInfo to file
fn write_info_file(info: &CdnInfo, filepath: &Path) {
    let mut local_info_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filepath)
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
    local_info_file
        .write_all(serde_json::to_string_pretty(&info).unwrap().as_bytes())
        .unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
}

fn file_get_sha1(path: &Path) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(&path).unwrap());
    sha1.digest().to_string()
}

fn copy_if_exists(origin: &Path, destination: &Path) {
    if origin.exists() {
        let file_dir = destination.parent().unwrap();

        if !file_dir.exists() {
            fs::create_dir_all(&file_dir).unwrap_or_else(|error| {
                panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
            });
        }

        fs::copy(&origin, destination).unwrap_or_else(|error| {
            panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
        });
    }
}

// as you may have noticed this seems to be well written code.
// as you may have guessed, i didnt write this code.
// https://stackoverflow.com/a/58063083
fn get_subdirs(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .collect())
}

fn get_backups(install_dir: &Path) -> Vec<PathBuf> {
    let mut backups = get_subdirs(&Path::join(install_dir, "backup")).unwrap_or_else(|_| vec![]);
    backups.sort();
    backups
}

fn backup(args: &args::Args, local_info: &CdnInfo) {
    let install_dir = Path::new(&args.directory);
    let backup_dir: PathBuf = [&args.directory, "backup", &local_info.revision.to_string()]
        .iter()
        .collect();

    // get existing backups
    let backups = get_backups(&install_dir);

    // delete everything but latest 3 backups.
    // hopefully above .sort will return dirs in the right order
    // otherwise we'll have to get directory timestamps
    if backups.len() > 3 {
        for backup in backups.iter().enumerate().take(backups.len() - 2) {
            println!("{:?}", backup.1);
            fs::remove_dir_all(backup.1).unwrap_or_else(|error| {
                panic!("\n\n{}:\n{:?}", "Error".bright_red(), error);
            });
        }
    }

    // copy files
    for file in &local_info.files {
        let curr_file_path = Path::join(install_dir, Path::new(&file.name));
        let backup_file_path = Path::join(&backup_dir, Path::new(&file.name));
        copy_if_exists(&curr_file_path, &backup_file_path);
    }

    // copy cdn_info.json
    let curr_cdn_info = Path::join(install_dir, "cdn_info.json");
    let backup_cdn_info = Path::join(&backup_dir, "cdn_info.json");
    copy_if_exists(&curr_cdn_info, &backup_cdn_info);
}

fn update(args: &args::Args, cdn_info: &CdnInfo, local_info: &CdnInfo) {
    let install_dir = Path::new(&args.directory);

    if !args.silent {
        println!(
            "Remote revision: {}",
            cdn_info.revision.to_string().bright_purple()
        );
    }

    // only update if remote version number is bigger than local
    // or if explicitly requested to update
    if (local_info.revision >= cdn_info.revision) && !args.force {
        if !args.silent {
            println!(
                "Local revision: {}",
                local_info.revision.to_string().green()
            )
        };
        return;
    }

    if !args.silent {
        println!(
            "Local revision: {}",
            local_info.revision.to_string().yellow()
        )
    };

    // keep track of processed files
    let mut stats = UpdateStats {
        checked: 0,
        downloaded: 0,
        skipped: 0,
    };

    if !args.no_backup {
        backup(args, local_info);
    }

    // iterate cdn files
    for cdn_file in &cdn_info.files {
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

    write_info_file(cdn_info, &Path::join(install_dir, "cdn_info.json"));

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

    let local_info = read_info_file(&Path::join(Path::new(&args.directory), "cdn_info.json"));
    let cdn_info = parse_info(&http::get_body_string(&args.cdn_url));

    if args.check {
        if cdn_info.revision > local_info.revision {
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    if args.backup_list {
        let backups = get_backups(Path::new(&args.directory));
        println!(
            "Installed version: {}",
            local_info.revision.to_string().green()
        );
        println!("Backups:");
        for b in backups {
            println!("      {}", b.file_name().unwrap().to_str().unwrap());
        }
        std::process::exit(0);
    }

    update(&args, &cdn_info, &local_info);
    std::process::exit(0);
}
