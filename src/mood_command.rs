use std::io;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;

use crate::daily_score;
use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;
use crate::Config;

pub struct MoodCommand {
    pub config: Config,
}

#[derive(Debug, PartialEq)]
pub enum MoodCommandError {
    CannotOpenFile { file_path: String, open_error: io::ErrorKind },
    CannotReadLine { file_path: String },
    DailyScoreParseError { line: String, daily_score_parse_error: daily_score::DailyScoreParseError },
}

impl MoodCommand {
    pub fn run(&self) -> Result<(), MoodCommandError> {
        let mut records = Vec::<DailyScore>::new();

        let file = OpenOptions::new()
            .read(true)
            .open(self.config.file_path.as_str())
            .map_err(|open_error| MoodCommandError::CannotOpenFile { file_path: self.config.file_path.clone(), open_error: open_error.kind() })?;

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line_string = line.map_err(|_| MoodCommandError::CannotReadLine { file_path: self.config.file_path.clone() })?;
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

        println!("30-days mood: {}", mood_report.thirty_days_mood());
        println!("365-days mood: {}", mood_report.yearly_mood());
        println!("30-days moving mood: {:?}", mood_report.thirty_days_moving_mood());

        Ok(())
    }
}
