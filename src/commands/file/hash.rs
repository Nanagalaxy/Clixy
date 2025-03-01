use std::{fs::File, io::Read, path::Path};

use clap::{Args, builder};
use hex::encode;

use crate::utils::hash::HashAlgorithm;

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
        default_value = "sha2-256",
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

        if !source_path.exists() || !source_path.is_file() {
            eprintln!("The source path does not exist or is not a file.");
            return;
        }

        println!("{}", self.algorithm);

        let Ok(mut file) = File::open(source_path) else {
            eprintln!("Error opening file.");
            return;
        };

        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_err() {
            eprintln!("Error reading file.");
            return;
        }

        let hash = encode(self.algorithm.compute(buffer));

        println!("{hash}");
    }
}
