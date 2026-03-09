mod bookmark;
mod commands;
mod forge;
mod protos;
mod store;

use clap::Parser;

use commands::cli::Cli;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    commands::run(cli)
}
