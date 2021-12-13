use std::{io, fmt};
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::error::Error;

use crate::daily_score;
use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;
use crate::Config;

pub struct MoodCommand {
    pub config: Config,
    pub report_type: MoodReportType,
}

pub enum MoodReportType {
    Monthly,
    Yearly,
    MovingMonthly,
}

#[derive(Debug)]//, PartialEq)]
pub enum MoodCommandError {
    CannotOpenFile { file_path: String, open_error: io::Error },
    CannotReadLine { file_path: String, read_error: io::Error },
    DailyScoreParseError { line: String, daily_score_parse_error: daily_score::ParseError },
}

impl std::error::Error for MoodCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotOpenFile { file_path: _, open_error } => Some(open_error),
            Self::CannotReadLine { file_path: _, read_error } => Some(read_error),
            Self::DailyScoreParseError { line: _, daily_score_parse_error } => Some(daily_score_parse_error),
        }
    }
}

impl fmt::Display for MoodCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotOpenFile { file_path, open_error: _ } => write!(f, "{} '{}'", "cannot open journal file", file_path),
            Self::CannotReadLine { file_path, read_error: _ } => write!(f, "{} '{}'", "cannot read journal file line", file_path),
            Self::DailyScoreParseError { line, daily_score_parse_error: _ } => write!(f, "{} '{}'", "cannot parse daily score data", line),
        }

        //    MoodCommandError::CannotOpenFile { file_path, open_error } => {
        //        let submessage = match open_error {
        //            io::ErrorKind::NotFound => "file not found".to_string(),
        //            io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        //            _ => "unknown error".to_string(),
        //        };
        //        format!("cannot open journal file '{}': {}", file_path, submessage)
        //    },
        //    MoodCommandError::CannotReadLine { file_path } => format!("cannot read journal file '{}': unknown error", file_path),
    }
}

impl MoodCommand {
    pub fn run(&self) -> Result<(), MoodCommandError> {
        let mut records = Vec::<DailyScore>::new();

        let file = OpenOptions::new()
            .read(true)
            .open(self.config.file_path.as_str())
            .map_err(|open_error| MoodCommandError::CannotOpenFile {
                file_path: self.config.file_path.clone(),
                open_error,
            })?;

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line_string = line.map_err(|read_error| MoodCommandError::CannotReadLine {
                file_path: self.config.file_path.clone(),
                read_error,
            })?;

            let daily_score =
                DailyScore::parse(line_string.as_str())
                .map_err(|daily_score_parse_error|
                         MoodCommandError::DailyScoreParseError {
                             line: line_string.clone(),
                             daily_score_parse_error,
                         })?;

            records.push(daily_score);
        }

        let mood_report = MoodReport::from_daily_scores(records);

        match self.report_type {
            MoodReportType::Monthly => println!("30-days mood: {}", mood_report.thirty_days_mood()),
            MoodReportType::Yearly => println!("365-days mood: {}", mood_report.yearly_mood()),
            MoodReportType::MovingMonthly => println!("30-days moving mood: {:?}", mood_report.thirty_days_moving_mood()),
        }

        Ok(())
    }
}
