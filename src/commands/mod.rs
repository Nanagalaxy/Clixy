use clap::{builder, Args, Subcommand};

pub mod copy;
pub mod r#move;
pub mod remove;

/// A struct that holds the options available for all commands.
#[derive(Args, Clone)]
pub struct BaseCmdOpt {
    #[arg(
        long,
        default_value = "10",
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Set the number of worker threads to use. Must be greater than 0. If an error occurs, the default value is used but the user must confirm the operation."
    )]
    workers: usize,
}

#[derive(Subcommand, Clone)]
pub enum FileCmd {
    #[command(about = "Copy the source path to the destination path")]
    Copy(copy::Command),

    #[command(about = "Remove the source path")]
    Remove(remove::Command),

    #[command(
        about = "Move the source path to the destination path. It's the same as copying the source path to the destination path and then removing the source path."
    )]
    Move(r#move::Command),
}
