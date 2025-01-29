use clap::Subcommand;

pub mod hash;

#[derive(Subcommand, Clone)]
#[command(about = "Cryptographic operations", visible_aliases = &["c"])]
pub enum CryptoCmd {
    #[command(about = "Hash the provided value", visible_aliases = &["h"])]
    Hash(hash::Command),
}
