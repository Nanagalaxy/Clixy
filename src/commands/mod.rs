use clap::{builder, Args};

pub mod crypto;
pub mod file;
pub mod random;

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
            "describe" => println!("This feature provides access to the describe command, which allows you to get detailed information about other features. (You're using it right now!)"),
            "file" => println!("This feature provides access to the copy, remove, and move commands. These commands allow you to copy, remove, or move files, giving you control over file management."),
            "random" => println!("This feature provides access to the random command, allowing you to generate random numbers, strings, and more."),
            _ => println!("The feature '{}' is not available or is not yet implemented.", self.feature),
        }
    }
}
