use clap::{Args, builder};
use rand::{
    distr::{Distribution, Uniform},
    rng,
};

#[derive(Args, Clone)]
pub struct Command {
    #[arg(
        long,
        default_value = "0",
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Minimum value of the random number."
    )]
    min: usize,

    #[arg(
        long,
        default_value = "100",
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Maximum value of the random number."
    )]
    max: usize,

    #[arg(
        short,
        long,
        default_value = "1",
        value_parser = builder::RangedU64ValueParser::<usize>::new(),
        help = "Number of times to repeat the command. Each number will be printed on a new line."
    )]
    repeat: usize,

    #[arg(
        short,
        long,
        default_value = "\n",
        value_parser = builder::StringValueParser::new(),
        help = "Separator to use between numbers."
    )]
    separator: String,
}

impl Command {
    pub fn execute(&self) {
        if self.min > self.max {
            eprintln!("The minimum value must be less than or equal to the maximum value.");
            return;
        }

        let Ok(range) = Uniform::new_inclusive(self.min, self.max) else {
            eprintln!("Error creating the random number generator.");
            return;
        };

        let mut rng = rng();

        for i in 0..self.repeat {
            print!("{}", range.sample(&mut rng));

            if self.repeat > 1 && i < self.repeat - 1 {
                print!("{}", self.separator);
            }
        }

        println!();
    }
}
