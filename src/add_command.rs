use chrono::prelude::*;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io;
use std::fmt;
use std::error::Error;

use crate::daily_score::DailyScore;
use crate::Config;

pub struct AddCommand {
    pub score: i8,
    pub datetime: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub config: Config,
}

#[derive(Debug)]
pub enum AddCommandError {
    CannotOpenFile { file_path: String, open_error: io::Error },
    CannotWriteToFile { file_path: String, write_error: io::Error },
}

impl std::error::Error for AddCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotOpenFile { file_path: _, open_error } => Some(open_error),
            Self::CannotWriteToFile { file_path: _, write_error } => Some(write_error),
        }
    }
}

impl fmt::Display for AddCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotOpenFile { file_path, open_error: _ } => write!(f, "cannot open journal file '{}'", file_path),
            Self::CannotWriteToFile { file_path, write_error: _ } => write!(f, "cannot write to journal file '{}'", file_path),
        }
    }
}

impl AddCommand {
    pub fn run(&self) -> Result<(), AddCommandError> {
        let daily_score = DailyScore {
            score: self.score,
            comment: self.comment.clone().unwrap_or_else(String::new),
            datetime: self.datetime.unwrap_or_else(Utc::now),
        };

        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(self.config.file_path.clone())
            .map_err(|open_error| AddCommandError::CannotOpenFile { file_path: self.config.file_path.clone(), open_error })?;

        writeln!(file, "{}", daily_score.to_s())
            .map_err(|write_error| AddCommandError::CannotWriteToFile { file_path: self.config.file_path.clone(), write_error })?;

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

        assert_eq!(AddCommandError::CannotOpenFile { file_path: file_path.clone(), open_error: io_error }.to_string(),
            "cannot open journal file 'path/to/file'");
        assert_eq!(AddCommandError::CannotWriteToFile { file_path, write_error: another_io_error }.to_string(),
            "cannot write to journal file 'path/to/file'");
    }
}
