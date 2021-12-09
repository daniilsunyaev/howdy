use std::fmt;
use std::io;
use std::num;

use crate::add_command::{AddCommand, AddCommandError};
use crate::mood_command::{MoodCommand, MoodCommandError};
use crate::daily_score::DailyScoreParseError;

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const JOURNAL_SEPARATOR: char = '|';

mod daily_score;
mod add_command;
mod mood_command;
mod mood_report;
mod test_helpers;

pub enum CommandSelectError {
    CommandNotProvided,
    CommandNotRecognized(String),
}

#[derive(Debug, PartialEq)]
pub struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<AddCommandError> for CliError {
    fn from(error: add_command::AddCommandError) -> Self {
        let message = match error {
            AddCommandError::MissingDailyScore => ("daily score is not provided").to_string(),
            AddCommandError::InvalidDailyScore { score_string, parse_error } => {
                let submessage = match parse_error {
                    num::IntErrorKind::InvalidDigit => format!("'{}' is not a valid integer", score_string),
                    num::IntErrorKind::PosOverflow =>  format!("'{}' is too big", score_string),
                    num::IntErrorKind::NegOverflow =>  format!("'{}' is too small", score_string),
                    _ => "unknown error".to_string(),
                };
                format!("failed to parse daily score: {}", submessage)
            },
            AddCommandError::CannotOpenFile { file_path, open_error } => {
                let submessage = match open_error {
                    io::ErrorKind::PermissionDenied => "permission denied".to_string(),
                    _ => "unknown error".to_string(),
                };
                format!("cannot open or create journal file '{}': {}", file_path, submessage)
            },
            AddCommandError::CannotWriteToFile { file_path, write_error } => {
                let submessage = match write_error {
                    io::ErrorKind::PermissionDenied => "permission denied".to_string(),
                    _ => "unknown error".to_string(),
                };
                format!("cannot write to journal file '{}': {}", file_path, submessage)
            }
        };

        Self(format!("add command failed: {}", message))
    }
}

impl From<MoodCommandError> for CliError {
    fn from(error: mood_command::MoodCommandError) -> Self {
        let message = match error {
            MoodCommandError::CannotOpenFile { file_path, open_error } => {
                let submessage = match open_error {
                    io::ErrorKind::NotFound => "file not found".to_string(),
                    io::ErrorKind::PermissionDenied => "permission denied".to_string(),
                    _ => "unknown error".to_string(),
                };
                format!("cannot open journal file '{}': {}", file_path, submessage)
            },
            MoodCommandError::CannotReadLine { file_path } => format!("cannot read journal file '{}': unknown error", file_path),
            MoodCommandError::DailyScoreParseError { line, daily_score_parse_error } => {
                let submessage = match daily_score_parse_error {
                    DailyScoreParseError::MissingDateTime => "datetime is missing".to_string(),
                    DailyScoreParseError::InvalidDateTime(date_string) => format!("'{}' is not a valid datetime", date_string),
                    DailyScoreParseError::MissingScore => "missing score".to_string(),
                    DailyScoreParseError::InvalidScore(score_string) => format!("'{}' is not a valid score", score_string),
                };
                format!("cannot parse daily score data '{}': {}", line, submessage)
            }
        };

        Self(format!("mood command failed: {}", message))
    }
}

impl From<CommandSelectError> for CliError {
    fn from(error: CommandSelectError) -> Self {
        let message = match error {
            CommandSelectError::CommandNotProvided => format!("command is not provided"),
            CommandSelectError::CommandNotRecognized(command) => format!("command '{}' is not recognized", command),
        };

        Self(message)
    }
}

pub fn run<I>(mut cli_args: I) -> Result<(), CliError>
    where
        I: Iterator<Item = String>,
    {
    cli_args.next(); // skip exec filename
    let command = cli_args.next().ok_or(CommandSelectError::CommandNotProvided)?;

    match command.as_str() {
        "add" => Ok(AddCommand::parse(cli_args)?.run()?),
        "mood" => Ok((MoodCommand {}).run()?),
        unrecognized_command => Err(CliError::from(CommandSelectError::CommandNotRecognized(unrecognized_command.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use std::num;
    use std::io;

    use crate::test_helpers::build_cli_args;
    use super::*;

    #[test]
    fn no_command_error() {
        let args = Vec::new();

        assert_eq!(run(args.into_iter()).unwrap_err(), CliError("command is not provided".to_string()));
    }

    #[test]
    fn wrong_command_error() {
        let args = build_cli_args("exec/path foo");

        assert_eq!(run(args).unwrap_err(), CliError("command 'foo' is not recognized".to_string()));
    }

    #[test]
    fn no_add_args_error() {
        let args = build_cli_args("exec/path add");

        assert_eq!(run(args.into_iter()).unwrap_err(),
            CliError("add command failed: daily score is not provided".to_string()));
    }

    #[test]
    fn add_command_error_consumption() {
        assert_eq!(CliError::from(AddCommandError::MissingDailyScore).0,
            "add command failed: daily score is not provided");

        assert_eq!(
            CliError::from(
                AddCommandError::InvalidDailyScore { score_string: "x".to_string(), parse_error: num::IntErrorKind::InvalidDigit }
            ).0,
            "add command failed: failed to parse daily score: 'x' is not a valid integer".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::InvalidDailyScore { score_string: "254".to_string(), parse_error: num::IntErrorKind::PosOverflow }
            ).0,
            "add command failed: failed to parse daily score: '254' is too big".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::InvalidDailyScore { score_string: "-250".to_string(), parse_error: num::IntErrorKind::NegOverflow }
            ).0,
            "add command failed: failed to parse daily score: '-250' is too small".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::PermissionDenied }
            ).0,
            "add command failed: cannot open or create journal file '~/path': permission denied".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::AddrInUse }
            ).0,
            "add command failed: cannot open or create journal file '~/path': unknown error".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotWriteToFile { file_path: "~/path".to_string(), write_error: io::ErrorKind::PermissionDenied }
            ).0,
            "add command failed: cannot write to journal file '~/path': permission denied".to_string()
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotWriteToFile { file_path: "~/path".to_string(), write_error: io::ErrorKind::Unsupported }
            ).0,
            "add command failed: cannot write to journal file '~/path': unknown error".to_string()
        );
    }

    #[test]
    fn mood_command_error_consumption() {
        assert_eq!(
            CliError::from(
                MoodCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::NotFound }
            ).0,
            "mood command failed: cannot open journal file '~/path': file not found".to_string()
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::PermissionDenied }
            ).0,
            "mood command failed: cannot open journal file '~/path': permission denied".to_string()
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::CannotReadLine { file_path: "~/path".to_string() }
            ).0,
            "mood command failed: cannot read journal file '~/path': unknown error".to_string()
        );


        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "11 | foo bar baz".to_string(),
                    daily_score_parse_error: DailyScoreParseError::InvalidDateTime("11".to_string()),
                }
            ).0,
            "mood command failed: cannot parse daily score data '11 | foo bar baz': '11' is not a valid datetime".to_string()
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "2020-02-01 09:10:11 +0000".to_string(),
                    daily_score_parse_error: DailyScoreParseError::MissingScore,
                }
            ).0,
            "mood command failed: cannot parse daily score data '2020-02-01 09:10:11 +0000': missing score".to_string()
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "2020-02-01 09:10:11 +0000 | foo |".to_string(),
                    daily_score_parse_error: DailyScoreParseError::InvalidScore("foo |".to_string()),
                }
            ).0,
            "mood command failed: cannot parse daily score data '2020-02-01 09:10:11 +0000 | foo |': 'foo |' is not a valid score".to_string()
        );
    }
}
