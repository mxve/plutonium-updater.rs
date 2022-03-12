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

    /// List backups
    #[clap(long)]
    pub backup_list: bool,

    #[clap(
        long,
        hide(true),
        default_value = "https://cdn.plutonium.pw/updater/prod/info.json"
    )]
    pub cdn_url: String,
}

pub fn get() -> Args {
    let mut args = Args::parse();
    args.directory = args.directory.replace("\"", "");
    args
}
