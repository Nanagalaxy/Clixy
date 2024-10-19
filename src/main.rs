#[allow(clippy::too_many_lines)]
mod commands;
mod path_content;
mod progress_bar_helper;
mod utils;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};

use commands::{
    copy::{self},
    r#move::{self},
    remove::{self},
};

#[derive(Parser)]
#[command(author = crate_authors!("\n"), version = crate_version!(), about = crate_description!())]
struct ArgsCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[command(about = "Copy the source path to the destination path")]
    Copy(copy::Command),

    #[command(about = "Remove the source path")]
    Remove(remove::Command),

    #[command(
        about = "Move the source path to the destination path. It's the same as copying the source path to the destination path and then removing the source path."
    )]
    Move(r#move::Command),
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        Commands::Copy(command) => {
            copy::execute(command);
        }
        Commands::Remove(command) => {
            remove::execute(command);
        }
        Commands::Move(command) => {
            r#move::execute(command);
        }
    }
}
