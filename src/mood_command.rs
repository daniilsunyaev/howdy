use std::io::prelude::*; // TODO: get rid of glob imports
use std::io::BufReader;
use std::fs::OpenOptions;
use std::fmt;

use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;

pub struct MoodCommand {
}

#[derive(Debug, PartialEq)]
pub enum MoodCommandError {
    CannotOpenFile,
    CannotReadLine,
    DailyScoreParseError,
}

impl fmt::Display for MoodCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MoodCommandError::CannotOpenFile => write!(f, "error opening journal file"),
            MoodCommandError::CannotReadLine => write!(f, "error reading the line"),
            MoodCommandError::DailyScoreParseError => write!(f, "error parsing the line"),
        }
    }
}

impl MoodCommand {
    pub fn run(&self) -> Result<(), MoodCommandError> {
        let mut records = Vec::<DailyScore>::new();

        let file = OpenOptions::new()
            .read(true)
            .open(crate::JOURNAL_FILE_PATH)
            .map_err(|_| MoodCommandError::CannotOpenFile)?; //TODO: do not omit io error

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let daily_score = DailyScore::parse(line
                                                .map_err(|_| MoodCommandError::CannotReadLine)?
                                                .as_str())
                .map_err(|_| MoodCommandError::DailyScoreParseError)?;

            records.push(daily_score);
        }

        let mood_report = MoodReport::from_daily_scores(records);

        println!("30-days mood: {}", mood_report.thirty_days_mood());
        println!("365-days mood: {}", mood_report.yearly_mood());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_ok() {
        let mood = MoodCommand {};

        assert_eq!(mood.run().is_err(), false);
    }

    #[test]
    fn error_descriptions() {
        assert_eq!(format!("{}", MoodCommandError::CannotOpenFile), "error opening journal file");
        assert_eq!(format!("{}", MoodCommandError::CannotReadLine), "error reading the line");
        assert_eq!(format!("{}", MoodCommandError::DailyScoreParseError), "error parsing the line");
    }
}
