use clap::Subcommand;

pub mod number;
pub mod string;

#[derive(Subcommand, Clone)]
#[command(about = "Generate random values", visible_aliases = &["rand"])]
pub enum RandomCmd {
    #[command(about = "Generate random strings", visible_aliases = &["str"])]
    String(string::Command),

    #[command(about = "Generate random numbers", visible_aliases = &["num"])]
    Number(number::Command),
}
