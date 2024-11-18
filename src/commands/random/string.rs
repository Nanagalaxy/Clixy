use clap::{builder, Args, ValueEnum};
use rand::distributions::{Alphanumeric, DistString, Distribution, Uniform};

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
            println!(
                "{}",
                Alphanumeric.sample_string(&mut rand::thread_rng(), self.size)
            );
            return;
        }

        static LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
        static UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        static NUMERIC: &str = "0123456789";
        static SPECIAL: &str = r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

        let mut charset = String::new();

        for c in &self.charsets {
            match c {
                Charset::Lower => charset.push_str(LOWERCASE),
                Charset::Upper => charset.push_str(UPPERCASE),
                Charset::Numeric => charset.push_str(NUMERIC),
                Charset::Special => charset.push_str(SPECIAL),
            }
        }

        if charset.is_empty() {
            eprintln!(
                "All character sets are excluded. Please include at least one character set."
            );
            return;
        }

        let charset = charset.as_bytes();

        let range = Uniform::new(0, charset.len());
        let mut rng = rand::thread_rng();

        for _ in 0..self.size {
            print!("{}", charset[range.sample(&mut rng)] as char);
        }

        println!();
    }
}
