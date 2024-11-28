use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Installation directory
    #[clap(short, long, default_value = "plutonium")]
    pub directory: String,

    /// Force file hash check, even if version matches
    #[clap(short, long)]
    pub force: bool,

    /// Download launcher assets
    #[clap(short, long)]
    pub launcher: bool,

    /// Hide file actions
    #[clap(short, long)]
    pub quiet: bool,

    /// Completely hide non-error output
    #[clap(short, long)]
    pub silent: bool,

    /// Check for update, returns exit code 0 for up to date and 1 for outdated
    #[clap(short, long)]
    pub check: bool,

    /// Disable colors
    #[clap(long)]
    pub no_color: bool,

    /// Create backup of current version while updating
    #[clap(long)]
    pub backup: bool,

    /// Create/update backup of current version
    #[clap(long)]
    pub manual_backup: bool,

    /// List backups
    #[clap(long)]
    pub backup_list: bool,

    /// Restore backup
    #[clap(long, default_value = "undefined")]
    pub backup_restore: String,

    /// Deprecated, backups are now disabled by default
    #[clap(long)]
    pub no_backup: bool,

    /// Override cdn url
    #[clap(
        long,
        default_value = "https://cdn.plutoniummod.com/updater/prod/info.json"
    )]
    pub cdn_url: String,

    /// get local version
    #[clap(long)]
    pub version_local: bool,

    // get cdn version
    #[clap(long)]
    pub version_cdn: bool,

    // Exclude remote dirs
    #[clap(short, long)]
    pub exclude: Vec<String>,

    /// Number of download threads
    #[clap(long, default_value = "2")]
    pub threads: usize,
}

pub fn get() -> Args {
    let mut args = Args::parse();
    args.directory = args.directory.replace('"', "");
    args
}
