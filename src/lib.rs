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

#[derive(Debug, PartialEq)]
pub enum CliError {
    CommandNotProvided,
    CommandNotRecognized(String),
    AddCommandArgsMissingDailyScore,
    AddCommandArgsInvalidDailyScore { score_string: String, parse_error: num::IntErrorKind },
    CommandExecutionError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            CliError::CommandNotProvided => format!("command is not provided"),
            CliError::CommandNotRecognized(command) => format!("command '{}' is not recognized", command),
            CliError::AddCommandArgsMissingDailyScore => "daily score is not provided for add command".to_string(),
            CliError::AddCommandArgsInvalidDailyScore { score_string, parse_error } => {
                let submessage = match parse_error {
                    num::IntErrorKind::InvalidDigit => format!("'{}' is not a valid integer", score_string),
                    num::IntErrorKind::PosOverflow =>  format!("'{}' is too big", score_string),
                    num::IntErrorKind::NegOverflow =>  format!("'{}' is too small", score_string),
                    _ => "unknown error".to_string(),
                };
                format!("failed to parse daily score for add command: {}", submessage)
            },
            CliError::CommandExecutionError(message) => message.to_string(),
        };
        write!(f, "{}", message)
    }
}

impl From<AddCommandError> for CliError {
    fn from(error: add_command::AddCommandError) -> Self {
        let message = match error {
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

        CliError::CommandExecutionError(format!("add command failed:\n{}", message))
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

        CliError::CommandExecutionError(format!("mood command failed:\n{}", message))
    }
}

fn parse_add_command_args<I>(mut args: I) -> Result<AddCommand, CliError>
    where
    I: Iterator<Item = String>,
{
    let score_string = args.next()
        .ok_or(CliError::AddCommandArgsMissingDailyScore)?;

    let score = score_string.parse::<i8>()
        .map_err(|parse_error| CliError::AddCommandArgsInvalidDailyScore {
            score_string: score_string,
            parse_error: parse_error.kind().clone(),
        })?;

    let comment: String = args.collect::<Vec<String>>().join(" ");

    return Ok(AddCommand { score, comment: Some(comment), datetime: None })
}


pub fn run<I>(mut cli_args: I) -> Result<(), CliError>
    where
        I: Iterator<Item = String>,
    {
    cli_args.next(); // skip exec filename
    let command = cli_args.next().ok_or(CliError::CommandNotProvided)?;

    match command.as_str() {
        "add" => Ok(parse_add_command_args(cli_args)?.run()?),
        "mood" => Ok((MoodCommand {}).run()?),
        unrecognized_command => Err(CliError::CommandNotRecognized(unrecognized_command.to_string())),
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
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err, CliError::CommandNotProvided);
        assert_eq!(format!("{}", result_err), "command is not provided".to_string());
    }

    #[test]
    fn wrong_command_error() {
        let args = build_cli_args("exec/path foo");
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err, CliError::CommandNotRecognized("foo".to_string()));
        assert_eq!(format!("{}", result_err), "command 'foo' is not recognized");
    }

    #[test]
    fn no_add_args_error() {
        let args = build_cli_args("exec/path add");
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err, CliError::AddCommandArgsMissingDailyScore);
        assert_eq!(format!("{}", result_err),
            "daily score is not provided for add command")
    }

    #[test]
    fn wrong_add_score_error() {
        let args = build_cli_args("exec/path add x");
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err,
            CliError::AddCommandArgsInvalidDailyScore {
                score_string: "x".to_string(),
                parse_error: num::IntErrorKind::InvalidDigit,
            });
        assert_eq!(format!("{}", result_err),
            "failed to parse daily score for add command: 'x' is not a valid integer".to_string())
    }

    #[test]
    fn big_add_score_error() {
        let args = build_cli_args("exec/path add 254");
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err,
            CliError::AddCommandArgsInvalidDailyScore {
                score_string: "254".to_string(),
                parse_error: num::IntErrorKind::PosOverflow,
            });

        assert_eq!(format!("{}", result_err),
            "failed to parse daily score for add command: '254' is too big".to_string());
    }

    #[test]
    fn small_add_score_error() {
        let args = build_cli_args("exec/path add -250");
        let result_err = run(args.into_iter()).err().unwrap();

        assert_eq!(result_err,
            CliError::AddCommandArgsInvalidDailyScore {
                score_string: "-250".to_string(),
                parse_error: num::IntErrorKind::NegOverflow,
            });

        assert_eq!(format!("{}", result_err),
            "failed to parse daily score for add command: '-250' is too small".to_string());
    }

    #[test]
    fn add_command_error_consumption() {
        assert_eq!(
            CliError::from(
                AddCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::PermissionDenied }
            ),
            CliError::CommandExecutionError(
                "add command failed:\ncannot open or create journal file '~/path': permission denied".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::AddrInUse }
            ),
            CliError::CommandExecutionError(
                "add command failed:\ncannot open or create journal file '~/path': unknown error".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotWriteToFile { file_path: "~/path".to_string(), write_error: io::ErrorKind::PermissionDenied }
            ),
            CliError::CommandExecutionError(
                "add command failed:\ncannot write to journal file '~/path': permission denied".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                AddCommandError::CannotWriteToFile { file_path: "~/path".to_string(), write_error: io::ErrorKind::Unsupported }
            ),
            CliError::CommandExecutionError(
                "add command failed:\ncannot write to journal file '~/path': unknown error".to_string()
            )
        );
    }

    #[test]
    fn mood_command_error_consumption() {
        assert_eq!(
            CliError::from(
                MoodCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::NotFound }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot open journal file '~/path': file not found".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::CannotOpenFile { file_path: "~/path".to_string(), open_error: io::ErrorKind::PermissionDenied }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot open journal file '~/path': permission denied".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::CannotReadLine { file_path: "~/path".to_string() }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot read journal file '~/path': unknown error".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "11 | foo bar baz".to_string(),
                    daily_score_parse_error: DailyScoreParseError::InvalidDateTime("11".to_string()),
                }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot parse daily score data '11 | foo bar baz': '11' is not a valid datetime".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "2020-02-01 09:10:11 +0000".to_string(),
                    daily_score_parse_error: DailyScoreParseError::MissingScore,
                }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot parse daily score data '2020-02-01 09:10:11 +0000': missing score".to_string()
            )
        );

        assert_eq!(
            CliError::from(
                MoodCommandError::DailyScoreParseError {
                    line: "2020-02-01 09:10:11 +0000 | foo |".to_string(),
                    daily_score_parse_error: DailyScoreParseError::InvalidScore("foo |".to_string()),
                }
            ),
            CliError::CommandExecutionError(
                "mood command failed:\ncannot parse daily score data '2020-02-01 09:10:11 +0000 | foo |': 'foo |' is not a valid score".to_string()
            )
        );
    }
}
