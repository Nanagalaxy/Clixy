mod commands;
mod path_content;
mod progress_bar_helper;
mod utils;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};

use commands::{
    copy::{execute_copy, CopyCommand},
    r#move::{execute_move, MoveCommand},
    remove::{execute_remove, RemoveCommand},
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
    Copy(CopyCommand),

    #[command(about = "Remove the source path")]
    Remove(RemoveCommand),

    #[command(
        about = "Move the source path to the destination path. It's the same as copying the source path to the destination path and then removing the source path."
    )]
    Move(MoveCommand),
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        Commands::Copy(command) => {
            execute_copy(command);
        }
        Commands::Remove(command) => {
            execute_remove(command);
        }
        Commands::Move(command) => {
            execute_move(command);
        }
    }
}
