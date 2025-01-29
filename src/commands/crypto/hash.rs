use clap::{builder, Args};
use hex::encode;

use crate::utils::hash::HashAlgorithm;

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The string to hash."
    )]
    value: String,

    #[arg(
        short,
        long,
        default_value = "md5",
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
        println!("{}", self.algorithm);

        let hash = encode(self.algorithm.compute(self.value.as_bytes()));

        println!("{hash}");
    }
}
