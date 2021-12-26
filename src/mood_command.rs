use std::{io, fmt};
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::error::Error;
use std::collections::HashSet;

use crate::daily_score;
use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;
use crate::Config;

mod plot;

pub struct MoodCommand {
    pub config: Config,
    pub report_type: MoodReportType,
    pub tags: HashSet<String>,
}

pub enum MoodReportType {
    WeeklyIterative,
    Monthly,
    Yearly,
    MovingMonthly,
}

#[derive(Debug)]
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
            Self::CannotOpenFile { file_path, open_error: _ } => write!(f, "cannot open journal file '{}'", file_path),
            Self::CannotReadLine { file_path, read_error: _ } => write!(f, "cannot read line from journal file '{}'", file_path),
            Self::DailyScoreParseError { line, daily_score_parse_error: _ } => write!(f, "cannot parse daily score data '{}'", line),
        }
    }
}

impl MoodCommand {
    pub fn run(self) -> Result<(), MoodCommandError> {
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

        let mood_report = MoodReport { daily_scores: &records, tags: &self.tags };

        // TODO: all println calls should be moved to the cli app level when code will be splitted
        // to core + cli app [+ gui app]
        match self.report_type {
            MoodReportType::Monthly => println!("30-days mood: {}", mood_report.thirty_days_mood()),
            MoodReportType::Yearly => println!("365-days mood: {}", mood_report.yearly_mood()),
            MoodReportType::WeeklyIterative => {
                let data = mood_report.iterative_weekly_mood();
                println!("weekly moods: {:?}", data.iter().map(|ts| ts.1).collect::<Vec<i32>>());
                if !data.is_empty() {
                    if let Err(error) = plot::draw(&data) {
                        println!("Warning: can't init gnuplot: {:?}", error);
                    };
                }
            },
            MoodReportType::MovingMonthly => {
                let data = mood_report.thirty_days_moving_mood();
                println!("30-days moving mood: {:?}", data.iter().map(|ts| ts.1).collect::<Vec<i32>>());
                if !data.is_empty() {
                    if let Err(error) = plot::draw(&data) {
                        println!("Warning: can't init gnuplot: {:?}", error);
                    };
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_display() {
        let file_path = String::from("path/to/file");
        let io_error = io::Error::new(io::ErrorKind::Other, "error text");
        let another_io_error = io::Error::new(io::ErrorKind::Other, "error text");
        let line = String::from("foo bar baz");
        let daily_score_parse_error = daily_score::ParseError::MissingDateTime;

        assert_eq!(MoodCommandError::CannotOpenFile { file_path: file_path.clone(), open_error: io_error }.to_string(),
            "cannot open journal file 'path/to/file'");
        assert_eq!(MoodCommandError::CannotReadLine { file_path, read_error: another_io_error }.to_string(),
            "cannot read line from journal file 'path/to/file'");
        assert_eq!(MoodCommandError::DailyScoreParseError { line, daily_score_parse_error }.to_string(),
            "cannot parse daily score data 'foo bar baz'");
    }
}
