pub mod completion;
pub mod crypto;
pub mod file;
pub mod random;

pub use completion::CompletionCmd;

use clap::{Args, ValueHint, builder};

/// A struct that holds the options available for all commands.
#[derive(Args, Clone)]
pub struct BaseCmdOpt {
    #[arg(
        long,
        default_value = "10",
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Set the number of worker threads to use. Must be greater than 0. If an error occurs, the default value is used but the user must confirm the operation.",
        value_hint = ValueHint::Other,
    )]
    workers: usize,
}

#[derive(Args, Clone)]
#[command(about = "Describe a feature", visible_aliases = &["d", "desc"])]
pub struct DescribeCmd {
    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The feature to describe. Available options: describe, crypto, file, random.",
        value_hint = ValueHint::Other,
    )]
    feature: String,
}

impl DescribeCmd {
    pub fn execute(&self) {
        match self.feature.trim().to_lowercase().as_str() {
            "describe" => println!(
                "This feature provides access to the describe command, which allows you to get detailed information about other features. (You're using it right now!)"
            ),
            "crypto" => println!(
                "This feature provides access to the crypto command, which allows you to encrypt and decrypt text or hash values."
            ),
            "file" => println!(
                "This feature provides access to the file command, which allows you to perform file operations or hashing."
            ),
            "random" => println!(
                "This feature provides access to the random command, allowing you to generate random numbers, strings, and more."
            ),
            _ => println!(
                "The feature '{}' is not available or is not yet implemented.",
                self.feature
            ),
        }
    }
}
