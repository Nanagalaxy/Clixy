use clap::{Subcommand, ValueEnum};

pub mod caesar;
pub mod hash;

#[derive(Subcommand, Clone)]
#[command(about = "Cryptographic operations", visible_aliases = &["c"])]
pub enum CryptoCmd {
    #[command(about = "Hash the provided value", visible_aliases = &["h"])]
    Hash(hash::Command),

    #[command(about = "Encrypt or decrypt a message using the Caesar cipher")]
    Caesar(caesar::Command),
}

#[derive(Debug, ValueEnum, Clone, PartialEq)]
enum Cipher {
    Encrypt,
    Decrypt,
}
