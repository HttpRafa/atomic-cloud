use clap::{ArgAction, Parser};

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, help = "Enable debug mode", action = ArgAction::SetTrue)]
    pub debug: bool,

    #[clap(
        help = "What command should the wrapper run",
        required = true,
        trailing_var_arg = true
    )]
    pub command: Vec<String>,
}
