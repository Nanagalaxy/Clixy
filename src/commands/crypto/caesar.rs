use clap::{builder, Args};
use deunicode::deunicode;

use super::Cipher;

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        value_enum,
        action = clap::ArgAction::Set,
        num_args = 1,
        ignore_case = true,
        help = "Specify the operation to perform."
    )]
    cipher: Cipher,

    #[arg(
        required = true,
        value_parser = builder::NonEmptyStringValueParser::new(),
        help = "The string to encrypt or decrypt."
    )]
    value: String,

    #[arg(
        short,
        long,
        default_value_t = 3,
        value_parser = builder::RangedI64ValueParser::<i64>::new(),
        allow_hyphen_values = true,
        help = "The shift value to use for the Caesar cipher."
    )]
    shift: i64,
}

impl Command {
    pub fn execute(&self) {
        let string = match self.cipher {
            Cipher::Encrypt => self.encrypt(),
            Cipher::Decrypt => self.decrypt(),
        };

        println!("{string}");
    }

    pub fn encrypt(&self) -> String {
        let value = deunicode(&self.value);

        #[allow(clippy::cast_possible_truncation)]
        // rem_euclid returns a value in the range of 0..26
        let shift = self.shift.rem_euclid(26) as u8;
        let mut result = String::new();

        for c in value.chars() {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let offset = c as u8 - base;
                let shifted = (offset + shift) % 26;
                result.push((shifted + base) as char);
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn decrypt(&self) -> String {
        let value = deunicode(&self.value);

        #[allow(clippy::cast_possible_truncation)]
        // rem_euclid returns a value in the range of 0..26
        let shift = self.shift.rem_euclid(26) as u8;
        let mut result = String::new();

        for c in value.chars() {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                let offset = c as u8 - base;
                let shifted = (offset + 26 - shift) % 26;
                result.push((shifted + base) as char);
            } else {
                result.push(c);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Hello, World!", 3, "Khoor, Zruog!")]
    #[case("Hello, World!", -3, "Ebiil, Tloia!")]
    #[case("Hello, World!", 26, "Hello, World!")]
    #[case("Hello, World!", 0, "Hello, World!")]
    fn test_encrypt(#[case] value: &str, #[case] shift: i64, #[case] expected: &str) {
        let command = Command {
            cipher: Cipher::Encrypt,
            value: value.to_string(),
            shift,
        };

        assert_eq!(command.encrypt(), expected);
    }

    #[rstest]
    #[case("Khoor, Zruog!", 3, "Hello, World!")]
    #[case("Ebiil, Tloia!", -3, "Hello, World!")]
    #[case("Hello, World!", 26, "Hello, World!")]
    #[case("Hello, World!", 0, "Hello, World!")]
    fn test_decrypt(#[case] value: &str, #[case] shift: i64, #[case] expected: &str) {
        let command = Command {
            cipher: Cipher::Decrypt,
            value: value.to_string(),
            shift,
        };

        assert_eq!(command.decrypt(), expected);
    }
}
