use clap::Subcommand;

pub mod copy;
pub mod r#move;
pub mod remove;

#[derive(Subcommand, Clone)]
pub enum FileCmd {
    #[command(about = "Copy the source path to the destination path")]
    Copy(copy::Command),

    #[command(about = "Remove the source path")]
    Remove(remove::Command),

    #[command(
        about = "Move the source path to the destination path. It's the same as copying the source path to the destination path and then removing the source path."
    )]
    Move(r#move::Command),
}
