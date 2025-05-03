use clap::{Args, ValueHint, builder};
use hex::encode;

use crate::utils::hash::HashAlgorithm;

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The string to hash.",
        value_hint = ValueHint::Other,
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
        help = "Specify the hash algorithm to use.",
        value_hint = ValueHint::Other,
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
