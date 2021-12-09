use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io;
use std::num;

use crate::daily_score::DailyScore;

pub struct AddCommand {
    daily_score: DailyScore,
}

#[derive(Debug, PartialEq)]
pub enum AddCommandError {
    MissingDailyScore,
    InvalidDailyScore { score_string: String, parse_error: num::IntErrorKind },
    CannotOpenFile { file_path: String, open_error: io::ErrorKind },
    CannotWriteToFile { file_path: String, write_error: io::ErrorKind },
}

impl AddCommand {
    pub fn parse<I>(mut args: I) -> Result<AddCommand, AddCommandError>
    where
        I: Iterator<Item = String>,
    {
        let score_string = args.next()
            .ok_or(AddCommandError::MissingDailyScore)?;

        let score = score_string.parse::<i8>()
            .map_err(|parse_error| AddCommandError::InvalidDailyScore { score_string: score_string, parse_error: parse_error.kind().clone() })?;

        let comment: String = args.collect::<Vec<String>>().join(" ");

        let daily_score = DailyScore { score, comment, datetime: Utc::now() };

        return Ok(AddCommand { daily_score })
    }

    pub fn run(&self) -> Result<(), AddCommandError> {
        let file_path = crate::JOURNAL_FILE_PATH.to_string();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(file_path.clone())
            .map_err(|open_error| AddCommandError::CannotOpenFile { file_path: file_path.clone(), open_error: open_error.kind() })?;

        writeln!(file, "{}", self.daily_score.to_s())
            .map_err(|write_error| AddCommandError::CannotWriteToFile { file_path, write_error: write_error.kind() })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::build_cli_args;
    use super::*;

    #[test]
    fn no_add_args_error() {
        let args = None;

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(), AddCommandError::MissingDailyScore);
    }

    #[test]
    fn wrong_score_error() {
        let args = build_cli_args("x");

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(),
            AddCommandError::InvalidDailyScore { score_string: "x".to_string(), parse_error: num::IntErrorKind::InvalidDigit });
    }

    #[test]
    fn big_score_error() {
        let args = build_cli_args("254");

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(),
            AddCommandError::InvalidDailyScore { score_string: "254".to_string(), parse_error: num::IntErrorKind::PosOverflow });
    }

    #[test]
    fn small_score_error() {
        let args = build_cli_args("-250");

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(),
            AddCommandError::InvalidDailyScore { score_string: "-250".to_string(), parse_error: num::IntErrorKind::NegOverflow });
    }

    #[test]
    fn correct_args() {
        let args = build_cli_args("-1 how  are you");
        let parsed = AddCommand::parse(args);

        assert_eq!(parsed.is_ok(), true);


        let command = parsed.unwrap();

        assert_eq!(command.daily_score.score, -1);
        assert_eq!(command.daily_score.comment, "how  are you");
    }
}
