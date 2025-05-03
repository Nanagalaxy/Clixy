use std::{
    fs::File,
    io::{self, BufWriter},
};

use crate::ArgsCli;

use clap::{Args, Command, CommandFactory, ValueEnum, ValueHint, builder};
use clap_complete::{Generator, Shell as ClapShell, generate};
use clap_complete_nushell::Nushell;

/// Workaround to use clap_complete::Shell as ClapShell and clap_complete_nushell::Nushell in the same arg
#[derive(Debug, ValueEnum, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
    Nushell,
}

#[derive(Args, Clone)]
#[command(about = "Generate shell completions", visible_aliases = &["cpl"])]
pub struct CompletionCmd {
    #[arg(
        required = true,
        value_parser = builder::EnumValueParser::<Shell>::new(),
        value_enum,
        ignore_case = true,
        help = "The shell to generate completions for.",
        value_hint = ValueHint::Other,
    )]
    shell: Shell,

    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The destination file to write the completions to.",
        value_hint = ValueHint::FilePath,
    )]
    destination: String,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Overwrite the destination file if it already exists."
    )]
    replace: bool,
}

impl CompletionCmd {
    pub fn execute(&self) {
        let mut cmd = ArgsCli::command();

        println!("Generating completions for {}...", cmd.get_name());

        let file = if self.replace {
            match File::create(&self.destination) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error creating file at {}: {}", self.destination, e);
                    return;
                }
            }
        } else {
            match File::create_new(&self.destination) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error creating file at {}: {}", self.destination, e);
                    return;
                }
            }
        };

        let mut buffer = BufWriter::new(file);

        match self.shell {
            Shell::Bash => Self::completion(ClapShell::Bash, &mut cmd, &mut buffer),
            Shell::Elvish => Self::completion(ClapShell::Elvish, &mut cmd, &mut buffer),
            Shell::Fish => Self::completion(ClapShell::Fish, &mut cmd, &mut buffer),
            Shell::Powershell => Self::completion(ClapShell::PowerShell, &mut cmd, &mut buffer),
            Shell::Zsh => Self::completion(ClapShell::Zsh, &mut cmd, &mut buffer),
            Shell::Nushell => Self::completion(Nushell, &mut cmd, &mut buffer),
        }

        println!("Completions generated successfully!");
    }

    fn completion<G: Generator>(generator: G, cmd: &mut Command, buffer: &mut dyn io::Write) {
        generate(generator, cmd, cmd.get_name().to_string(), buffer);
    }
}
