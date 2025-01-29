#[allow(clippy::too_many_lines)]
mod commands;
mod path_content;
mod progress_bar_helper;
mod utils;

use clap::{crate_authors, crate_description, crate_version, Parser, Subcommand};

use commands::{
    file::{copy, r#move, remove, FileCmd},
    random::RandomCmd,
    DescribeCmd,
};

#[derive(Parser)]
#[command(
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
    after_help = ArgsCli::after_help()
)]
struct ArgsCli {
    #[command(subcommand)]
    command: Commands,
}

impl ArgsCli {
    fn after_help() -> String {
        let features = [
            ("Describe:", cfg!(feature = "describe")),
            ("File:", cfg!(feature = "file")),
            ("Random:", cfg!(feature = "random")),
        ];

        let max_lenght = features
            .iter()
            .map(|(feature, _)| feature.len())
            .max()
            .unwrap_or(0);

        let mut help_text = String::from("Enabled features:\n");
        for (feature, enabled) in &features {
            help_text.push_str(&format!("    {feature:<max_lenght$} {enabled}\n"));
        }

        help_text
    }
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[cfg(feature = "describe")]
    #[command(about = "Describe a feature", visible_aliases = &["d", "desc"])]
    Describe(DescribeCmd),

    #[cfg(feature = "file")]
    #[command(subcommand)]
    File(FileCmd),

    #[cfg(feature = "random")]
    #[command(subcommand)]
    Random(RandomCmd),
}

fn main() {
    let args = ArgsCli::parse();

    match args.command {
        #[cfg(feature = "describe")]
        Commands::Describe(command) => {
            command.execute();
        }
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
            FileCmd::Hash(command) => {
                command.execute();
            }
        },
        #[cfg(feature = "random")]
        Commands::Random(command) => match command {
            RandomCmd::String(command) => {
                command.execute();
            }
            RandomCmd::Number(command) => {
                command.execute();
            }
        },
    }
}
