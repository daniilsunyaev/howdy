use std::{fmt, num};
use std::error::Error;
use std::ops::Deref;
use std::collections::HashSet;

use crate::add_command::{AddCommand, AddCommandError};
use crate::mood_command::{MoodCommand, MoodReportType, MoodCommandError};
use crate::export_command::{ExportCommand, ExportType, ExportCommandError};

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const XLSX_FILE_PATH: &str = "./howdy_journal.xlsx";
const JOURNAL_SEPARATOR: char = '|';
const TAGS_SEPARATOR: &str = ",";

mod daily_score;
mod add_command;
mod mood_command;
mod export_command;
mod mood_report;
mod journal;
mod test_helpers;

#[derive(Debug)]
pub enum CliError {
    CommandNotProvided,
    FilenameNotProvided,
    CommandNotRecognized(String),
    AddCommandArgsMissingDailyScore,
    AddCommandArgsInvalidDailyScore { score_string: String, parse_error: num::ParseIntError },
    MoodReportTypeInvalid(String),
    CommandExecutionError(Box<dyn Error>),
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandExecutionError(error) => Some(error.deref()),
            Self::AddCommandArgsInvalidDailyScore { score_string: _, parse_error } => Some(parse_error),
            _ => None
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::CommandNotProvided => "command is not provided".to_string(),
            Self::FilenameNotProvided => "'-f' option requires file path which is not provided".to_string(),
            Self::CommandNotRecognized(command) => format!("command '{}' is not recognized", command),
            Self::AddCommandArgsMissingDailyScore => "daily score is not provided for add command".to_string(),
            Self::AddCommandArgsInvalidDailyScore { score_string, parse_error: _ } => {
                format!("cannot parse daily score '{}' as int for add command", score_string)
            },
            Self::MoodReportTypeInvalid(report_type) => format!("'{}' is not a valid mood report type", report_type),
            Self::CommandExecutionError(_) => "failed to execute command".to_string(),
        };
        write!(f, "{}", message)
    }
}

impl From<AddCommandError> for CliError {
    fn from(error: add_command::AddCommandError) -> Self {
        Self::CommandExecutionError(Box::new(error))
    }
}

impl From<MoodCommandError> for CliError {
    fn from(error: mood_command::MoodCommandError) -> Self {
        Self::CommandExecutionError(Box::new(error))
    }
}

impl From<ExportCommandError> for CliError {
    fn from(error: export_command::ExportCommandError) -> Self {
        Self::CommandExecutionError(Box::new(error))
    }
}

pub struct GlobalConfig {
    pub journal_file_path: String,
}

fn build_add_command<I>(mut args: I, global_config: GlobalConfig) -> Result<AddCommand, CliError>
    where
    I: Iterator<Item = String>,
{
    let mut tag_or_comment_sign;
    let mut tags = HashSet::new();

    let score_string = args.next()
        .ok_or(CliError::AddCommandArgsMissingDailyScore)?;

    let score = score_string.parse::<i8>()
        .map_err(|parse_error| CliError::AddCommandArgsInvalidDailyScore { score_string, parse_error })?;

    loop {
        tag_or_comment_sign = args.next();
        match tag_or_comment_sign.as_deref() {
            Some("--comment") | Some("-c") | None => break,
            Some(tag) => tags.insert(tag.to_string())
        };
    };

    let comment_string: String = args.collect::<Vec<String>>().join(" ");
    let comment = if comment_string.is_empty() {
        None
    } else {
        Some(comment_string)
    };

    Ok(AddCommand { score, tags, comment: comment, datetime: None, global_config })
}

fn build_mood_command<I>(mut args: I, global_config: GlobalConfig) -> Result<MoodCommand, CliError>
    where
    I: Iterator<Item = String>,
{
    let mut tag_or_type_sign;
    let mut tags = HashSet::new();
    loop {
        tag_or_type_sign = args.next();
        match tag_or_type_sign.as_deref() {
            Some("--type") | Some("-t") | None => break,
            Some(tag) => tags.insert(tag.to_string())
        };
    };

    let report_type_str = args.next();
    let report_type = match report_type_str.as_deref() {
        Some("m") | Some("monthly") => MoodReportType::MonthlyIterative,
        Some("lm") | Some("last month") => MoodReportType::Monthly,
        Some("ly") | Some("last year") => MoodReportType::Yearly,
        Some("mm") | Some("moving") => MoodReportType::MovingMonthly,
        Some("w") | Some("weekly") => MoodReportType::WeeklyIterative,
        Some("7d") | Some("7 days") => MoodReportType::SevenDaysIterative,
        Some("30d") | Some("30 days") => MoodReportType::ThirtyDaysIterative,
        None => MoodReportType::Monthly,
        Some(unrecognized_option) => return Err(CliError::MoodReportTypeInvalid(unrecognized_option.to_string())),
    };

    Ok(MoodCommand { report_type, global_config, tags })
}

fn build_export_command<I>(mut args: I, global_config: GlobalConfig) -> Result<ExportCommand, CliError>
    where
    I: Iterator<Item = String>,
{
    let file_path = args.next().unwrap_or(XLSX_FILE_PATH.to_string());
    let export_type = ExportType::Xlsx;

    Ok(ExportCommand { global_config, export_type, file_path })
}

pub fn run<I>(mut cli_args: I) -> Result<(), CliError>
where
    I: Iterator<Item = String>,
{
    // skip exec filename
    cli_args.next();

    let mut global_config = GlobalConfig { journal_file_path: JOURNAL_FILE_PATH.to_string() };

    let mut argument = cli_args.next().ok_or(CliError::CommandNotProvided)?;
    if argument.as_str() == "-f" {
        global_config.journal_file_path = cli_args.next().ok_or(CliError::FilenameNotProvided)?;
        argument = cli_args.next().ok_or(CliError::CommandNotProvided)?
    };

    match argument.as_str() {
        "add" => build_add_command(cli_args, global_config)?.run()?,
        "mood" => build_mood_command(cli_args, global_config)?.run()?,
        "export" => build_export_command(cli_args, global_config)?.run()?,
        unrecognized_command => return Err(CliError::CommandNotRecognized(unrecognized_command.to_string())),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::build_cli_args;
    use super::*;

    #[test]
    fn no_command_error() {
        let args = Vec::new();
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::CommandNotProvided)
        );
        assert_eq!(format!("{}", result_err), "command is not provided".to_string());
    }

    #[test]
    fn no_file_path_error() {
        let args = build_cli_args("exec/path -f");
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::FilenameNotProvided)
        );
        assert_eq!(format!("{}", result_err), "'-f' option requires file path which is not provided");
    }

    #[test]
    fn wrong_command_error() {
        let args = build_cli_args("exec/path foo");
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::CommandNotRecognized(_))
        );
        assert_eq!(format!("{}", result_err), "command 'foo' is not recognized");
    }

    #[test]
    fn no_add_args_error() {
        let args = build_cli_args("exec/path add");
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::AddCommandArgsMissingDailyScore)
        );
        assert_eq!(format!("{}", result_err),
            "daily score is not provided for add command")
    }

    #[test]
    fn small_add_score_error() {
        let args = build_cli_args("exec/path add -250");
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::AddCommandArgsInvalidDailyScore { .. })
        );
        assert_eq!(format!("{}", result_err),
            "cannot parse daily score '-250' as int for add command".to_string());
    }

    #[test]
    fn wrong_mood_report_type_error() {
        let args = build_cli_args("exec/path mood -t mmm");
        let result_err = run(args.into_iter()).err().unwrap();

        assert!(
            matches!(result_err, CliError::MoodReportTypeInvalid(_))
        );

        assert_eq!(format!("{}", result_err),
            "'mmm' is not a valid mood report type".to_string());
    }
}
