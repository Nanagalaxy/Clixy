use clap::Subcommand;

pub mod copy;
pub mod r#move;
pub mod remove;
pub mod hash;

#[derive(Subcommand, Clone)]
#[command(about = "File operations", visible_aliases = &["f"])]
pub enum FileCmd {
    #[command(about = "Copy the source path to the destination path", visible_aliases = &["cp"])]
    Copy(copy::Command),

    #[command(about = "Remove the source path", visible_aliases = &["rm"])]
    Remove(remove::Command),

    #[command(
        about = "Move the source path to the destination path. Same as copy then remove.",
        visible_aliases = &["mv"]
    )]
    Move(r#move::Command),

    #[command(about = "Hash the source path", visible_aliases = &["h"])]
    Hash(hash::Command),
}
