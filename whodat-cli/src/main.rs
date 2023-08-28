use clap::{arg, command, Parser, Subcommand};
use whodat;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Shows information about a given device
    Show {},
}

fn main() {
    let cli = Cli::parse();
}
