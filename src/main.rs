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
    FileCmd,
};

#[derive(Parser)]
#[command(author = crate_authors!("\n"), version = crate_version!(), about = crate_description!())]
struct ArgsCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[cfg(feature = "file")]
    #[clap(flatten)]
    File(FileCmd),
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        #[cfg(feature = "file")]
        Commands::File(command) => match command {
            FileCmd::Copy(cmd) => {
                copy::execute(cmd);
            }
            FileCmd::Remove(cmd) => {
                remove::execute(cmd);
            }
            FileCmd::Move(cmd) => {
                r#move::execute(cmd);
            }
        },
    }
}
