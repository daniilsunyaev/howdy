use std::error::Error;
use std::fmt;

use crate::GlobalConfig;
use crate::journal;

pub struct ExportCommand {
    pub global_config: GlobalConfig,
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
        let daily_scores = journal::read(&self.global_config.journal_file_path)
            .map_err(ExportCommandError::ReadError)?;

        match self.export_type {
            ExportType::Xlsx =>  {
                journal::write_xlsx(&self.file_path, &daily_scores)
                    .map_err(ExportCommandError::WriteError)?;
                println!("Export to '{}' done", self.file_path);
            },
        }

        Ok(())
    }
}
