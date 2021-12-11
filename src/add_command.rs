use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io;

use crate::daily_score::DailyScore;
use crate::Config;

pub struct AddCommand {
    pub score: i8,
    pub datetime: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub config: Config,
}

#[derive(Debug, PartialEq)]
pub enum AddCommandError {
    CannotOpenFile { file_path: String, open_error: io::ErrorKind },
    CannotWriteToFile { file_path: String, write_error: io::ErrorKind },
}

impl AddCommand {
    pub fn run(&self) -> Result<(), AddCommandError> {
        let daily_score = DailyScore {
            score: self.score,
            comment: self.comment.clone().unwrap_or("".to_string()),
            datetime: self.datetime.unwrap_or(Utc::now()),
        };

        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(self.config.file_path.clone())
            .map_err(|open_error| AddCommandError::CannotOpenFile { file_path: self.config.file_path.clone(), open_error: open_error.kind() })?;

        writeln!(file, "{}", daily_score.to_s())
            .map_err(|write_error| AddCommandError::CannotWriteToFile { file_path: self.config.file_path.clone(), write_error: write_error.kind() })?;

        Ok(())
    }
}
