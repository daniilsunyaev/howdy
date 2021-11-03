use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fmt;

use crate::daily_score::DailyScore;

pub struct AddCommand {
    daily_score: DailyScore,
}

#[derive(Debug, PartialEq)]
pub enum AddCommandError {
    MissingDailyScore,
    InvalidDailyScore,
    CannotOpenFile,
    CannotWriteToFile,
}

impl fmt::Display for AddCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddCommandError::MissingDailyScore => write!(f, "failed to get daily score"),
            AddCommandError::InvalidDailyScore => write!(f, "failed to parse daily score"),
            AddCommandError::CannotOpenFile => write!(f, "cannot open or create journal file"),
            AddCommandError::CannotWriteToFile => write!(f, "cannot write to journal file"),
        }
    }
}

impl AddCommand {
    pub fn parse<I>(mut args: I) -> Result<AddCommand, AddCommandError>
    where
        I: Iterator<Item = String>,
    {
        let score = args.next()
            .ok_or(AddCommandError::MissingDailyScore)?
            .parse::<i8>()
            .map_err(|_| AddCommandError::InvalidDailyScore)?;

        let comment: String = args.collect::<Vec<String>>().join(" ");

        let daily_score = DailyScore { score, comment, datetime: Utc::now() };

        return Ok(AddCommand { daily_score })
    }

    pub fn run(&self) -> Result<(), AddCommandError> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(crate::JOURNAL_FILE_PATH)
            .map_err(|_| AddCommandError::CannotOpenFile)?;

        writeln!(file, "{}", self.daily_score.to_s())
            .map_err(|_| AddCommandError::CannotWriteToFile)?;

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
    fn no_add_args_error_description() {
        assert_eq!(format!("{}", AddCommandError::MissingDailyScore), "failed to get daily score");
    }

    #[test]
    fn no_add_args_error() {
        let args = None;

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(), AddCommandError::MissingDailyScore);
    }

    #[test]
    fn wrong_score_error_description() {
        assert_eq!(format!("{}", AddCommandError::InvalidDailyScore), "failed to parse daily score");
    }

    #[test]
    fn wrong_score_error() {
        let args = build_args("x");

        assert_eq!(AddCommand::parse(args.into_iter()).err().unwrap(), AddCommandError::InvalidDailyScore);
    }

    #[test]
    fn cannot_open_file_error_description() {
        assert_eq!(format!("{}", AddCommandError::CannotOpenFile), "cannot open or create journal file");
    }

    #[test]
    fn cannot_write_to_file_error_description() {
        assert_eq!(format!("{}", AddCommandError::CannotWriteToFile), "cannot write to journal file");
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
