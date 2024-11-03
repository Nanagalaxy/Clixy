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

#[derive(Args, Clone)]
pub struct DescribeCmd {
    #[arg(
        short,
        long,
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The feature to describe."
    )]
    feature: String,
}

impl DescribeCmd {
    pub fn execute(&self) {
        match self.feature.to_lowercase().as_str() {
            "describe" => println!("This feature gives access to the `describe` command, which allows you to describe a feature (you are using it right now)."),
            "file" => println!("This feature gives access to the `copy`, `remove`, and `move` commands, which allow you to copy, remove, and move files, respectively."),
            // "random" => println!("This feature gives access to the `random` command, which allows you to generate random numbers."),
            _ => println!("The feature '{}' is not yet available.", self.feature),
        }
    }
}
