use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Mutex},
    time::Instant,
};

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use nanoserde::{DeJson, SerJson};
use rayon::prelude::*;

mod args;
mod http;

#[derive(DeJson, SerJson)]
struct CdnInfo {
    product: String,
    revision: u16,
    #[nserde(rename = "baseUrl")]
    base_url: String,
    files: Vec<CdnFile>,
}

#[derive(DeJson, SerJson)]
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
    bytes_to_download: u64,
    bytes_downloaded: u64,
}

type Result<T> = std::result::Result<T, String>;

fn read_info_file(filepath: &Path) -> Result<CdnInfo> {
    let info_file = fs::read_to_string(filepath)
        .unwrap_or_else(|_| include_str!("assets/default_info.json").to_string());

    DeJson::deserialize_json(&info_file)
        .map_err(|e| format!("Failed to parse info file {}: {:#?}", filepath.display(), e))
}

fn write_info_file(info: &CdnInfo, filepath: &Path) -> Result<()> {
    fs::write(filepath, SerJson::serialize_json(info))
        .map_err(|e| format!("Failed to write info file {}: {:#?}", filepath.display(), e))
}

fn file_get_sha1(path: &Path) -> Result<String> {
    let content =
        fs::read(path).map_err(|e| format!("Failed to read file {}: {:#?}", path.display(), e))?;

    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&content);
    Ok(sha1.digest().to_string())
}

fn copy_if_exists(origin: &Path, destination: &Path) -> Result<()> {
    if origin.exists() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create directory {}: {:#?}", parent.display(), e)
            })?;
        }

        fs::copy(origin, destination).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {:#?}",
                origin.display(),
                destination.display(),
                e
            )
        })?;
    }
    Ok(())
}

fn get_subdirs(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?;
    Ok(entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect())
}

fn get_backups(install_dir: &Path) -> Vec<u16> {
    let mut backups: Vec<u16> = get_subdirs(&Path::join(install_dir, "backup"))
        .unwrap_or_default()
        .iter()
        .filter_map(|path| path.file_name()?.to_str()?.parse::<u16>().ok())
        .collect::<Vec<u16>>();
    backups.sort_unstable();
    backups
}

fn copy_version(info: &CdnInfo, source_dir: &Path, destination_dir: &Path) -> Result<()> {
    for file in &info.files {
        let source_file_path = source_dir.join(&file.name);
        let dest_file_path = destination_dir.join(&file.name);
        copy_if_exists(&source_file_path, &dest_file_path)?;
    }

    let source_file_path = source_dir.join("cdn_info.json");
    let dest_file_path = destination_dir.join("cdn_info.json");
    copy_if_exists(&source_file_path, &dest_file_path)?;
    Ok(())
}

fn backup(args: &args::Args, local_info: &CdnInfo, delete: bool) -> Result<()> {
    let install_dir = Path::new(&args.directory);
    let backup_dir: PathBuf = [&args.directory, "backup", &local_info.revision.to_string()]
        .iter()
        .collect();

    if delete {
        let backups = get_backups(install_dir);

        if backups.len() > 3 {
            for backup in backups.iter().enumerate().take(backups.len() - 2) {
                let backup_path: PathBuf = [&args.directory, "backup", &backup.1.to_string()]
                    .iter()
                    .collect();

                println!("Removing old backup {}", backup_path.display());

                fs::remove_dir_all(&backup_path).map_err(|e| {
                    format!(
                        "Failed to remove backup directory {}: {:#?}",
                        backup_path.display(),
                        e
                    )
                })?;
            }
        }
    }

    copy_version(local_info, install_dir, &backup_dir)?;
    Ok(())
}

fn restore_backup(args: args::Args) -> Result<()> {
    let install_dir = Path::new(&args.directory);
    let backup_dir: PathBuf = [&args.directory, "backup", &args.backup_restore]
        .iter()
        .collect();

    if backup_dir.exists() {
        let backup_info = read_info_file(&Path::join(&backup_dir, "cdn_info.json"))?;
        copy_version(&backup_info, &backup_dir, install_dir)?;
        Ok(())
    } else {
        Err(format!("Backup <{}> not found", &args.backup_restore))
    }
}

fn create_progress_bar(total: u64, silent: bool) -> ProgressBar {
    if silent {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.magenta} [{bar:.magenta}] >{msg:.green}<")
                .unwrap()
                .progress_chars("##-"),
        );
        pb
    }
}

fn should_skip_file(file: &CdnFile, args: &args::Args) -> bool {
    file.name.starts_with("launcher") && !args.launcher
}

fn handle_skipped_file(
    pb: &ProgressBar,
    file: &CdnFile,
    stats: &mut UpdateStats,
    args: &args::Args,
) {
    if !args.quiet && !args.silent {
        pb.println(format!("{}: {}", "Skipped".bright_blue(), &file.name));
    }
    stats.skipped += 1;
    pb.inc(file.size as u64);
}

fn handle_checked_file(
    pb: &ProgressBar,
    file: &CdnFile,
    stats: &mut UpdateStats,
    args: &args::Args,
) {
    if !args.quiet && !args.silent {
        pb.println(format!("{}: {}", "Checked".cyan(), file.name));
    }
    stats.checked += 1;
    pb.inc(file.size as u64);
}

fn handle_downloaded_file(
    pb: &ProgressBar,
    file: &CdnFile,
    stats: &mut UpdateStats,
    args: &args::Args,
) {
    if !args.quiet && !args.silent {
        pb.println(format!("{}: {}", "Downloaded".bright_yellow(), &file.name));
    }
    stats.downloaded += 1;
    stats.bytes_downloaded += file.size as u64;
    pb.inc(file.size as u64);
}

fn update(args: &args::Args, cdn_info: &CdnInfo, local_info: &CdnInfo) -> Result<()> {
    let install_dir = Path::new(&args.directory);

    if !args.silent {
        println!(
            "Remote revision: {}",
            cdn_info.revision.to_string().bright_purple()
        );
    }

    let needs_update = args.force || cdn_info.revision > local_info.revision;
    if !needs_update {
        if !args.silent {
            println!(
                "Local revision: {}",
                local_info.revision.to_string().green()
            );
        }
        return Ok(());
    }

    if !args.silent {
        println!(
            "Local revision: {}",
            local_info.revision.to_string().yellow()
        );
    }

    let stats = Arc::new(Mutex::new(UpdateStats {
        checked: 0,
        downloaded: 0,
        skipped: 0,
        bytes_to_download: cdn_info.files.iter().map(|f| f.size as u64).sum(),
        bytes_downloaded: 0,
    }));

    if args.backup {
        backup(args, local_info, true)?;
    }

    let pb = Arc::new(create_progress_bar(
        stats.lock().unwrap().bytes_to_download,
        args.silent,
    ));

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .map_err(|e| format!("Failed to initialize thread pool: {:#?}", e))?;

    let files: Vec<_> = cdn_info.files.iter().collect();
    let results: Vec<Result<()>> = files
        .par_iter()
        .map(|cdn_file| {
            let pb = Arc::clone(&pb);
            let stats = Arc::clone(&stats);

            if !args.silent {
                pb.set_message(cdn_file.name.to_string());
            }

            if should_skip_file(cdn_file, args) {
                handle_skipped_file(&pb, cdn_file, &mut stats.lock().unwrap(), args);
                return Ok(());
            }

            for exclude in &args.exclude {
                if cdn_file.name.starts_with(exclude) {
                    handle_skipped_file(&pb, cdn_file, &mut stats.lock().unwrap(), args);
                    return Ok(());
                }
            }

            let file_path = install_dir.join(&cdn_file.name);

            if file_path.exists() && file_get_sha1(&file_path)? == cdn_file.hash {
                handle_checked_file(&pb, cdn_file, &mut stats.lock().unwrap(), args);
                return Ok(());
            }

            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {}: {:#?}", parent.display(), e)
                })?;
            }

            http::download_file(
                &format!("{}{}", &cdn_info.base_url, &cdn_file.hash),
                &file_path,
            )?;

            handle_downloaded_file(&pb, cdn_file, &mut stats.lock().unwrap(), args);
            Ok(())
        })
        .collect();

    for result in results {
        result?;
    }

    write_info_file(cdn_info, &install_dir.join("cdn_info.json"))?;

    pb.finish_and_clear();

    if !args.silent {
        let stats = stats.lock().unwrap();
        println!(
            "{} hashes checked, {} files skipped, {} files ({}MB) downloaded",
            stats.checked,
            stats.skipped,
            stats.downloaded,
            stats.bytes_downloaded / 1024 / 1024
        );
    }

    Ok(())
}

#[cfg(windows)]
fn setup_env(no_color: bool) -> Result<()> {
    if no_color {
        colored::control::SHOULD_COLORIZE.set_override(false);
    } else {
        colored::control::set_virtual_terminal(true)
            .map_err(|e| format!("Failed to setup Windows terminal colors: {:#?}", e))?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn setup_env(no_color: bool) -> Result<()> {
    if no_color {
        colored::control::SHOULD_COLORIZE.set_override(false);
    }
    Ok(())
}

fn run() -> Result<()> {
    let start_time = Instant::now();
    let args = args::get();
    setup_env(args.no_color)?;

    let cdn = args.cdn_url.to_string();
    let local_info = read_info_file(&Path::join(Path::new(&args.directory), "cdn_info.json"))?;
    let cdn_body = http::get_body_string(&cdn)?;
    let cdn_info: CdnInfo = DeJson::deserialize_json(&cdn_body)
        .map_err(|e| format!("Failed to parse CDN response: {:#?}", e))?;

    if args.version_local {
        println!("{}", local_info.revision);
        return Ok(());
    }

    if args.version_cdn {
        println!("{}", cdn_info.revision);
        return Ok(());
    }

    if args.check {
        process::exit(if cdn_info.revision > local_info.revision {
            1
        } else {
            0
        });
    }

    if args.backup_list {
        let backups = get_backups(Path::new(&args.directory));
        println!(
            "Installed version: {}",
            local_info.revision.to_string().green()
        );
        println!("Backups:");
        for b in backups {
            println!("      {}", b);
        }
        return Ok(());
    }

    if args.backup_restore != "undefined" {
        backup(&args, &local_info, false)?;
        restore_backup(args)?;
        return Ok(());
    }

    if args.manual_backup {
        backup(&args, &local_info, false)?;
        return Ok(());
    }

    update(&args, &cdn_info, &local_info)?;

    if !args.silent {
        let duration = start_time.elapsed();
        println!("Total time: {:.2}s", duration.as_secs_f64());
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n\n{}:\n{}", "Error".bright_red(), e);
        process::exit(1);
    }
}
