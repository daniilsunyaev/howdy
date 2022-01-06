//use std::{io, fmt};
//use std::io::{BufRead, BufReader};
//use std::fs::OpenOptions;
use std::error::Error;
use std::collections::HashSet;
use std::fmt;

//use crate::daily_score;
//use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;
use crate::Config;
use crate::journal;

mod plot;

pub struct MoodCommand {
    pub config: Config,
    pub report_type: MoodReportType,
    pub tags: HashSet<String>,
}

pub enum MoodReportType {
    WeeklyIterative,
    SevenDaysIterative,
    Monthly,
    MonthlyIterative,
    ThirtyDaysIterative,
    Yearly,
    MovingMonthly,
}

impl MoodReportType {
    fn is_plottable(&self) -> bool {
        matches!(self, Self::WeeklyIterative | Self::SevenDaysIterative | Self::MonthlyIterative |
                 Self::ThirtyDaysIterative | Self::MovingMonthly)
    }
}

#[derive(Debug)]
pub enum MoodCommandError {
    JournalReadError(journal::JournalError),
}

impl std::error::Error for MoodCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::JournalReadError(journal_error) => Some(journal_error),
         }
    }
}

impl fmt::Display for MoodCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::JournalReadError(_journal_error) => write!(f, "cannot parse journal"),
        }
    }
}

impl MoodCommand {
    pub fn run(self) -> Result<(), MoodCommandError> {
        let daily_scores = journal::read(&self.config.file_path)
            .map_err(|journal_error| MoodCommandError::JournalReadError(journal_error))?;

        let mood_report = MoodReport { daily_scores: &daily_scores, tags: &self.tags };

        let (caption, data) = match self.report_type {
            MoodReportType::Monthly => ("30-days mood:", mood_report.thirty_days_mood()),
            MoodReportType::Yearly => ("365-days mood:", mood_report.yearly_mood()),
            MoodReportType::MonthlyIterative => ("monthly moods:", mood_report.iterative_monthly_mood()),
            MoodReportType::WeeklyIterative => ("weekly moods:",  mood_report.iterative_weekly_mood()),
            MoodReportType::SevenDaysIterative => ("weekly moods:", mood_report.iterative_seven_days_mood()),
            MoodReportType::ThirtyDaysIterative => ("thirty day intervals moods:", mood_report.iterative_thirty_days_mood()),
            MoodReportType::MovingMonthly => ("30-days moving mood:", mood_report.thirty_days_moving_mood()),
        };

        println!("{} {:?}", caption, data.iter().map(|ts| ts.1).collect::<Vec<i32>>());

        if self.report_type.is_plottable() && !data.is_empty() {
            if let Err(error) = plot::draw(&data) {
                println!("Warning: can't init gnuplot: {:?}", error);
            };
        }

        Ok(())
    }
}
