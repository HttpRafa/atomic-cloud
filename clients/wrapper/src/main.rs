use clap::Parser;
use cli::Cli;

mod cli;

fn main() {
    let cli = Cli::parse();
}