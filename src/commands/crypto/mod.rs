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
    #[value(help = "Encrypt a message using the Caesar cipher")]
    Encrypt,
    #[value(help = "Decrypt a message using the Caesar cipher")]
    Decrypt,
}
