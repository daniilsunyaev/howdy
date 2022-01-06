use std::error::Error;
use std::fmt;

use crate::Config;
use crate::journal;

pub struct ExportCommand {
    pub config: Config,
    pub export_type: ExportType,
    pub file_path: String,
}

pub enum ExportType {
    Xlsx,
}

#[derive(Debug)]
pub enum ExportCommandError {
    ReadError(journal::JournalError),
    WriteError(journal::JournalError),
}

impl std::error::Error for ExportCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadError(journal_error) => Some(journal_error),
            Self::WriteError(journal_error) => Some(journal_error),
         }
    }
}

impl fmt::Display for ExportCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadError(_journal_error) => write!(f, "cannot parse journal"),
            Self::WriteError(_journal_error) => write!(f, "cannot write to a file"),
        }
    }
}

impl ExportCommand {
    pub fn run(self) -> Result<(), ExportCommandError> {
        let daily_scores = journal::read(&self.config.file_path)
            .map_err(|journal_error| ExportCommandError::ReadError(journal_error))?;

        match self.export_type {
            ExportType::Xlsx =>  {
                journal::write_xlsx(&self.file_path, &daily_scores)
                    .map_err(|journal_error| ExportCommandError::WriteError(journal_error))?;
                println!("Export to '{}' done", self.file_path);
            },
        }

        Ok(())
    }
}
