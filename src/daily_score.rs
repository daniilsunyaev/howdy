use chrono::prelude::{DateTime, FixedOffset};
use std::fmt;

#[cfg(test)]
use chrono::prelude::Utc;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub struct DailyScore {
    pub score: i8,
    pub tags: Vec<String>,
    pub comment: String,
    pub datetime: DateTime<FixedOffset>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingDateTime,
    InvalidDateTime(String),
    MissingScore,
    InvalidScore(String),
}

impl std::error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            ParseError::MissingDateTime => "datetime is missing".to_string(),
            ParseError::InvalidDateTime(date_string) => format!("'{}' is not a valid datetime", date_string),
            ParseError::MissingScore => "missing score".to_string(),
            ParseError::InvalidScore(score_string) => format!("'{}' is not a valid score", score_string),
        };

        write!(f, "{}", message)
    }
}

impl DailyScore {
    #[cfg(test)]
    pub fn new() -> Self {
        Self { score: 0, comment: "".to_string(), tags: Vec::new(), datetime: Utc::now().into() }
    }

    #[cfg(test)]
    pub fn with_score(score: i8) -> Self {
        Self { score, ..Self::new() }
    }

    pub fn to_s(&self) -> String {
        format!("{} {} {} {} {} {} {}",
                self.datetime.format(DATE_FORMAT), crate::JOURNAL_SEPARATOR,
                self.score, crate::JOURNAL_SEPARATOR,
                self.tags.join(crate::TAGS_SEPARATOR), crate::JOURNAL_SEPARATOR,
                self.comment)
    }

    pub fn parse(daily_score_string: &str) -> Result<Self, ParseError> {
        let spaced_separator = format!(" {} ", crate::JOURNAL_SEPARATOR);
        let mut slice = daily_score_string.splitn(4, spaced_separator.as_str());

        let datetime_str = slice.next()
            .ok_or(ParseError::MissingDateTime)?;

        let datetime = DateTime::parse_from_str(datetime_str, DATE_FORMAT)
            .map_err(|_| ParseError::InvalidDateTime(datetime_str.to_string()))?;

        let score_str = slice.next()
            .ok_or(ParseError::MissingScore)?;

        let score = score_str.parse::<i8>()
            .map_err(|_| ParseError::InvalidScore(score_str.to_string()))?;

        let tags: Vec<String>;
        let tags_str = slice.next().unwrap_or("");
        if tags_str.len() > 0 {
            tags = tags_str.split(',').map(str::to_string).collect();
        } else {
            tags = vec![];
        }

        let comment = slice.next().unwrap_or("").to_string();
        Ok(DailyScore { score, tags, comment, datetime })
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::TimeZone;

    use super::*;

    #[test]
    fn string_formatting() {
        let local_date = FixedOffset::east(4 * 3600).ymd(2020, 1, 1).and_hms(9, 10, 11);
        let score = DailyScore {
            score: 1,
            comment: "foo || bar".to_string(),
            tags: vec!["run".to_string(), "games".to_string()],
            datetime: local_date.into()
        };

        assert_eq!(score.to_s(), "2020-01-01 09:10:11 +0400 | 1 | run,games | foo || bar")
    }

    #[test]
    fn string_parsing() {
        let daily_score_string = "2020-02-01 09:10:11 +0200 | 1 |  | foo || bar";
        let daily_score_parse_result = DailyScore::parse(daily_score_string);

        assert!(daily_score_parse_result.is_ok());

        let daily_score = daily_score_parse_result.unwrap();
        assert_eq!(daily_score.score, 1);
        assert_eq!(daily_score.comment, "foo || bar");
        assert_eq!(daily_score.tags, Vec::<String>::new());
        assert_eq!(Utc.ymd(2020, 2, 1).and_hms(7, 10, 11), daily_score.datetime);
    }

    #[test]
    fn invalid_string_parsing() {
        assert_eq!(DailyScore::parse("").err().unwrap(), ParseError::InvalidDateTime("".to_string()));
        assert_eq!(DailyScore::parse("fooo").err().unwrap(),
            ParseError::InvalidDateTime("fooo".to_string()));

        assert_eq!(DailyScore::parse("foo|").err().unwrap(),
            ParseError::InvalidDateTime("foo|".to_string()));

        assert_eq!(DailyScore::parse("2020-02-01 09:10:11 +0000|").err().unwrap(),
            ParseError::InvalidDateTime("2020-02-01 09:10:11 +0000|".to_string()));

        assert_eq!(DailyScore::parse("2020-02-01 09:10:11 +0000").err().unwrap(), ParseError::MissingScore);
        assert_eq!(DailyScore::parse("2020-02-01 09:10:11 +0000 | ").err().unwrap(),
            ParseError::InvalidScore("".to_string()));

        assert_eq!(DailyScore::parse("2020-02-01 09:10:11 +0000 | foo").err().unwrap(),
            ParseError::InvalidScore("foo".to_string()));

        assert_eq!(DailyScore::parse("2020-02-01 09:10:11 +0000 | 4|").err().unwrap(),
            ParseError::InvalidScore("4|".to_string()));
    }

    #[test]
    fn new_defaults() {
        let daily_score = DailyScore::new();

        assert_eq!(daily_score.score, 0);
        assert_eq!(daily_score.tags, Vec::<String>::new());
        assert_eq!(daily_score.comment, "");
    }

    #[test]
    fn new_with_score() {
        let daily_score = DailyScore::with_score(5);

        assert_eq!(daily_score.score, 5);
        assert_eq!(daily_score.tags, Vec::<String>::new());
        assert_eq!(daily_score.comment, "");
    }

    #[test]
    fn errors_display() {
        assert_eq!(ParseError::MissingDateTime.to_string(), "datetime is missing");
        assert_eq!(ParseError::InvalidDateTime("foo bar".to_string()).to_string(), "'foo bar' is not a valid datetime");
        assert_eq!(ParseError::MissingScore.to_string(), "missing score");
        assert_eq!(ParseError::InvalidScore("foo".to_string()).to_string(), "'foo' is not a valid score");
    }
}
