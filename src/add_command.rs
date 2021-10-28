use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;
use crate::daily_score::DailyScore;

pub struct AddCommand {
    daily_score: DailyScore,
}

impl AddCommand {
    pub fn parse<I>(mut args: I) -> Result<AddCommand, &'static str>
    where
        I: Iterator<Item = String>,
    {
        let score_string = match args.next() {
            Some(arg) => arg,
            None => return Err("failed to get daily score"),
        };
        let score: i8 = match score_string.parse() {
            Ok(int_score) => int_score,
            Err(_message) => return Err("failed to parse daily score"),
        };

        let comment: String = args.collect::<Vec<String>>().join(" ");

        let daily_score = DailyScore { score, comment, datetime: Utc::now() };

        return Ok(AddCommand { daily_score })
    }

    pub fn run(&self) -> Result<(), &'static str> {
        let open_journal = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(crate::JOURNAL_FILE_PATH);

        match open_journal {
            Err(message) => println!("error opening or creating journal file: {}", message), // TODO: use return Err instead
            Ok(mut file) => {
                println!("opened file successfully!");
                if let Err(message) = writeln!(file, "{}", self.daily_score.to_s()) {
                    println!("error appending to a file: {}", message);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_args(args_str: &str) -> impl Iterator<Item = String> + '_ {
        args_str.split(' ').map(|s| s.to_string())
    }


    #[test]
    fn no_add_args_error() {
        let args = None;

        assert_eq!(AddCommand::parse(args.into_iter()).err(), Some("failed to get daily score"));
    }

    #[test]
    fn wrong_score_error() {
        let args = build_args("x");

        assert_eq!(AddCommand::parse(args.into_iter()).err(), Some("failed to parse daily score"));
    }

    #[test]
    fn correct_args() {
        let args = build_args("-1 how  are you");
        let parsed = AddCommand::parse(args);

        assert_eq!(parsed.is_ok(), true);


        let command = parsed.unwrap();

        assert_eq!(command.daily_score.score, -1);
        assert_eq!(command.daily_score.comment, "how  are you");
    }
}
