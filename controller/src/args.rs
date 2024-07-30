use clap::{ArgAction, Parser};

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, help = "Enable debug mode", action = ArgAction::SetTrue)]
    pub debug: bool,
}
