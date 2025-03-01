use clap::{Args, ValueEnum, builder};
use rand::{
    distr::{Alphanumeric, Distribution, SampleString, Uniform},
    rng,
};

use crate::utils::alphabet::{ALPHABET_LOWER, ALPHABET_UPPER, NUMERIC, SPECIAL};

#[derive(ValueEnum, Clone, PartialEq)]
enum Charset {
    Lower,
    Upper,
    Numeric,
    Special,
}

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        short,
        long,
        default_value_t = 20,
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Size of the string to generate."
    )]
    size: usize,

    #[arg(
        short,
        long = "charset",
        value_enum,
        action = clap::ArgAction::Append,
        num_args(1..),
        help = "Specify one or more character sets to include. \
                If no sets are specified, the default set is alphanumeric (a-z, A-Z and 0-9)."
    )]
    charsets: Vec<Charset>,
}

impl Command {
    pub fn execute(&self) {
        if self.charsets.is_empty() {
            println!("{}", Alphanumeric.sample_string(&mut rng(), self.size));
            return;
        }

        let mut charset = Vec::new();

        for c in &self.charsets {
            match c {
                Charset::Lower => charset.extend(ALPHABET_LOWER),
                Charset::Upper => charset.extend(ALPHABET_UPPER),
                Charset::Numeric => charset.extend(NUMERIC),
                Charset::Special => charset.extend(SPECIAL),
            }
        }

        if charset.is_empty() {
            eprintln!(
                "All character sets are excluded. Please include at least one character set."
            );
            return;
        }

        let Ok(range) = Uniform::new(0, charset.len()) else {
            eprintln!("Failed to create random generator.");
            return;
        };

        let mut rng = rng();

        for _ in 0..self.size {
            print!("{}", charset[range.sample(&mut rng)] as char);
        }

        println!();
    }
}
