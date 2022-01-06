use simple_excel_writer as excel;
use excel::{row, Row, Workbook, Column};

use std::fs::OpenOptions;
use crate::daily_score;
use crate::daily_score::DailyScore;
use std::{io, fmt};
use std::io::{BufRead, BufReader};
use std::error::Error;

#[derive(Debug)]
pub enum JournalError {
    CannotOpenFile { file_path: String, open_error: io::Error },
    CannotReadLine { file_path: String, read_error: io::Error },
    DailyScoreParseError { line: String, daily_score_parse_error: daily_score::ParseError },
    XlsxWriteError(io::Error),
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotOpenFile { file_path, open_error: _ } => write!(f, "cannot open journal file '{}'", file_path),
            Self::CannotReadLine { file_path, read_error: _ } => write!(f, "cannot read line from journal file '{}'", file_path),
            Self::DailyScoreParseError { line, daily_score_parse_error: _ } => write!(f, "cannot parse daily score data '{}'", line),
            Self::XlsxWriteError(_) => write!(f, "cannot write to xlsx file"),
        }
    }
}

impl std::error::Error for JournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotOpenFile { file_path: _, open_error } => Some(open_error),
            Self::CannotReadLine { file_path: _, read_error } => Some(read_error),
            Self::DailyScoreParseError { line: _, daily_score_parse_error } => Some(daily_score_parse_error),
            Self::XlsxWriteError(error) => Some(error),
        }
    }
}

pub fn read(file_path: &str) -> Result<Vec<DailyScore>, JournalError> {
    let mut records = Vec::<DailyScore>::new();

    let file = OpenOptions::new()
        .read(true)
        .open(file_path)
        .map_err(|open_error| JournalError::CannotOpenFile {
            file_path: file_path.to_string(),
            open_error,
        })?;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line_string = line.map_err(|read_error| JournalError::CannotReadLine {
            file_path: file_path.to_string(),
            read_error,
        })?;

        let daily_score =
            DailyScore::parse(line_string.as_str())
            .map_err(|daily_score_parse_error|
                     JournalError::DailyScoreParseError {
                         line: line_string.clone(),
                         daily_score_parse_error,
                     })?;

        records.push(daily_score);
    }
    Ok(records)
}

pub fn write_xlsx(file_path: &str, daily_scores: &[DailyScore]) -> Result<(), JournalError> {
    let mut wb = Workbook::create(file_path);
    let mut sheet = wb.create_sheet("Daily Scores");
    sheet.add_column(Column { width: 10.0 });
    sheet.add_column(Column { width: 5.0 });
    sheet.add_column(Column { width: 40.0 });
    sheet.add_column(Column { width: 50.0 });

    wb.write_sheet(&mut sheet, |sheet_writer| {
        sheet_writer.append_row(row!["Date", "Score", "Tags", "Comment"])?;

        for daily_score in daily_scores.iter() {
            sheet_writer
                .append_row(row![
                            daily_score.datetime.date().naive_local(),
                            daily_score.score as f64,
                            daily_score.tags_string(),
                            daily_score.comment.as_deref().unwrap_or("")
                ])?
        };
        Ok(())
    }).map_err(JournalError::XlsxWriteError)?;

    Ok(())
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

        assert_eq!(JournalError::CannotOpenFile { file_path: file_path.clone(), open_error: io_error }.to_string(),
            "cannot open journal file 'path/to/file'");
        assert_eq!(JournalError::CannotReadLine { file_path, read_error: another_io_error }.to_string(),
            "cannot read line from journal file 'path/to/file'");
        assert_eq!(JournalError::DailyScoreParseError { line, daily_score_parse_error }.to_string(),
            "cannot parse daily score data 'foo bar baz'");
    }
}
