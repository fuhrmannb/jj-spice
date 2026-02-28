use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Submit,
}
