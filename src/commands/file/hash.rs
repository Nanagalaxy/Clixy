use std::{fmt::Display, path::Path};

use clap::{builder, Args, ValueEnum};
use hex::encode;

use crate::utils::{
    calculate_hash_md5, calculate_hash_sha1, calculate_hash_sha2_256, calculate_hash_sha2_512,
    calculate_hash_sha3_256, calculate_hash_sha3_512,
};

#[derive(Debug, ValueEnum, Clone, PartialEq)]
enum HashAlgorithm {
    Md5,
    Sha1,
    Sha2_256,
    Sha2_512,
    Sha3_256,
    Sha3_512,
}

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "MD5"),
            HashAlgorithm::Sha1 => write!(f, "SHA1"),
            HashAlgorithm::Sha2_256 => write!(f, "SHA2-256"),
            HashAlgorithm::Sha2_512 => write!(f, "SHA2-512"),
            HashAlgorithm::Sha3_256 => write!(f, "SHA3-256"),
            HashAlgorithm::Sha3_512 => write!(f, "SHA3-512"),
        }
    }
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

        if !source_path.exists() {
            eprintln!("The source path does not exist.");
            return;
        }

        println!("{}", self.algorithm);

        let hash = match self.algorithm {
            HashAlgorithm::Md5 => calculate_hash_md5(source_path),
            HashAlgorithm::Sha1 => calculate_hash_sha1(source_path),
            HashAlgorithm::Sha2_256 => calculate_hash_sha2_256(source_path),
            HashAlgorithm::Sha2_512 => calculate_hash_sha2_512(source_path),
            HashAlgorithm::Sha3_256 => calculate_hash_sha3_256(source_path),
            HashAlgorithm::Sha3_512 => calculate_hash_sha3_512(source_path),
        };

        let result_string = match hash {
            Ok(hash) => encode(hash),
            Err(_) => "Error calculating hash.".to_string(),
        };

        println!("{}", result_string);
    }
}
