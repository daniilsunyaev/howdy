use std::fmt;

use crate::add_command::{AddCommand, AddCommandError};
use crate::mood_command::{MoodCommand, MoodCommandError};

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const JOURNAL_SEPARATOR: char = '|';

mod daily_score;
mod add_command;
mod mood_command;
mod mood_report;

#[derive(Debug, PartialEq)]
pub enum CliError {
    AddCommandError(AddCommandError),
    MoodCommandError(MoodCommandError),
    CommandNotProvided,
    CommandNotRecognized,
}

impl From<AddCommandError> for CliError {
    fn from(error: AddCommandError) -> Self {
        CliError::AddCommandError(error)
    }
}

impl From<MoodCommandError> for CliError {
    fn from(error: MoodCommandError) -> Self {
        CliError::MoodCommandError(error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::AddCommandError(e) => write!(f, "add command failed: {}", e),
            CliError::MoodCommandError(e) => write!(f, "mood command failed: {}", e),
            CliError::CommandNotProvided => write!(f, "command is not provided"),
            CliError::CommandNotRecognized => write!(f, "command is not recognized"),
        }
    }
}


pub fn run<I>(mut cli_args: I) -> Result<(), CliError>
    where
        I: Iterator<Item = String>,
    {
    cli_args.next(); // skip exec filename
    let command = cli_args.next().ok_or(CliError::CommandNotProvided)?;

    match command.as_str() {
        "add" => Ok(AddCommand::parse(cli_args)?.run()?),
        "mood" => Ok((MoodCommand {}).run()?),
        _ => Err(CliError::CommandNotRecognized),
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_args(args_str: &str) -> impl Iterator<Item = String> + '_ {
        args_str.split(' ').map(|s| s.to_string())
    }

    #[test]
    fn no_command_error() {
        let args = Vec::new();

        assert_eq!(run(args.into_iter()).err(), Some(CliError::CommandNotProvided));
    }

    #[test]
    fn no_command_error_description() {
        assert_eq!(format!("{}", CliError::CommandNotProvided), "command is not provided");
    }

    #[test]
    fn wrong_command_error() {
        let args = build_args("exec/path foo");

        assert_eq!(run(args).err(), Some(CliError::CommandNotRecognized));
    }

    #[test]
    fn wrong_command_error_description() {
        assert_eq!(format!("{}", CliError::CommandNotRecognized), "command is not recognized");
    }

    #[test]
    fn no_add_args_error() {
        let args = build_args("exec/path add");

        assert_eq!(run(args.into_iter()).err(), Some(CliError::AddCommandError(AddCommandError::MissingDailyScore)));
        assert_eq!(format!("{}", CliError::AddCommandError(AddCommandError::MissingDailyScore)),
            "add command failed: failed to get daily score"
        );
    }
}
