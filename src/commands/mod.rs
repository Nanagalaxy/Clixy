use clap::{builder, Args};

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
