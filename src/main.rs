mod commands;
mod content_tree;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};

use commands::{
    copy::{execute_copy, CopyCommand},
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
    }
}
