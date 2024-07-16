use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[command(group(ArgGroup::new("features").multiple(true).args(&["enable_transfers", "enable_screens"])))]
pub struct Cli {
    #[arg(long, group = "features")]
    pub enable_transfers: bool,
    #[arg(long, group = "features")]
    pub enable_screens: bool,

    #[arg(long, group = "features")]
    pub transfer_command: Option<String>,

    #[arg(short, long)]
    pub program: String,
    #[arg(short, long)]
    pub args: Option<String>,
}