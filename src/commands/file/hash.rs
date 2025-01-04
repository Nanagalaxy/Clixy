use std::path::Path;

use clap::{builder, Args, ValueEnum};
use hex::encode;

use crate::utils::calculate_hash;

#[derive(Debug, ValueEnum, Clone, PartialEq)]
enum HashAlgorithm {
    Sha256,
}

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The source path to hash."
    )]
    source: String,

    #[arg(
        short,
        long,
        default_value = "sha256",
        value_enum,
        action = clap::ArgAction::Set,
        num_args = 1,
        ignore_case = true,
        help = "Specify the hash algorithm to use."
    )]
    algorithm: HashAlgorithm,
}

impl Command {
    pub fn execute(&self) {
        let source_path = Path::new(&self.source);

        if !source_path.exists() {
            eprintln!("The source path does not exist.");
            return;
        }

        println!("{:?}", self.algorithm);

        let hash = match self.algorithm {
            HashAlgorithm::Sha256 => calculate_hash(source_path),
            _ => calculate_hash(source_path),
        };

        let result_string = match hash {
            Ok(hash) => encode(hash),
            Err(_) => "Error calculating hash.".to_string(),
        };

        println!("{}", result_string);
    }
}
