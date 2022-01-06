use chrono::prelude::{DateTime, FixedOffset};
use std::fmt;
use std::collections::HashSet;

#[cfg(test)]
use chrono::prelude::Utc;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub struct DailyScore {
    pub score: i8,
    pub tags: HashSet<String>,
    pub comment: Option<String>,
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
        Self { score: 0, comment: None, tags: HashSet::new(), datetime: Utc::now().into() }
    }

    #[cfg(test)]
    pub fn with_score(score: i8) -> Self {
        Self { score, ..Self::new() }
    }

    pub fn to_s(&self) -> String {
        let comment_string = match &self.comment {
            Some(comment_string) => format!(" {}", comment_string),
            None => "".to_string(),
        };

        format!("{} {} {} {} {} {}{}",
                self.datetime.format(DATE_FORMAT),
                crate::JOURNAL_SEPARATOR,
                self.score,
                crate::JOURNAL_SEPARATOR,
                self.tags_string(),
                crate::JOURNAL_SEPARATOR,
                comment_string
        )
    }

    pub fn parse(daily_score_string: &str) -> Result<Self, ParseError> {
        let spaced_separator = format!(" {}", crate::JOURNAL_SEPARATOR);
        let mut slice = daily_score_string.splitn(4, &spaced_separator).map(str::trim);

        let datetime_str = slice.next()
            .ok_or(ParseError::MissingDateTime)?;

        let datetime = DateTime::parse_from_str(datetime_str, DATE_FORMAT)
            .map_err(|_| ParseError::InvalidDateTime(datetime_str.to_string()))?;

        let score_str = slice.next()
            .ok_or(ParseError::MissingScore)?;

        let score = score_str.parse::<i8>()
            .map_err(|_| ParseError::InvalidScore(score_str.to_string()))?;

        let tags_str = slice.next().unwrap_or("");
        let tags = if tags_str.is_empty() {
            HashSet::new()
        } else {
            tags_str.split(',').map(str::to_string).collect()
        };

        let comment = slice.next().map(str::to_string);
        Ok(DailyScore { score, tags, comment, datetime })
    }

    pub fn tags_string(&self) -> String {
        let mut tags_vec = self.tags.iter().map(String::as_str).collect::<Vec<&str>>();
        tags_vec.sort_unstable();
        tags_vec.join(crate::TAGS_SEPARATOR)
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::TimeZone;

    use super::*;

    #[test]
    fn tags_formatting() {
        let local_date = FixedOffset::east(4 * 3600).ymd(2020, 1, 1).and_hms(9, 10, 11);
        let score1 = DailyScore {
            score: 1,
            comment: Some("foo || bar".to_string()),
            tags: vec!["run".to_string(), "games".to_string()].into_iter().collect(),
            datetime: local_date.into()
        };

        let score2 = DailyScore {
            score: 1,
            comment: None,
            tags: HashSet::new(),
            datetime: local_date.into()
        };

        let score3 = DailyScore {
            score: 1,
            comment: None,
            tags: vec!["run".to_string()].into_iter().collect(),
            datetime: local_date.into()
        };

        assert_eq!(score1.tags_string(), "games,run");
        assert_eq!(score2.tags_string(), "");
        assert_eq!(score3.tags_string(), "run");
    }

    #[test]
    fn string_formatting() {
        let local_date = FixedOffset::east(4 * 3600).ymd(2020, 1, 1).and_hms(9, 10, 11);
        let score = DailyScore {
            score: 1,
            comment: Some("foo || bar".to_string()),
            tags: vec!["run".to_string(), "games".to_string()].into_iter().collect(),
            datetime: local_date.into()
        };

        assert_eq!(score.to_s(), "2020-01-01 09:10:11 +0400 | 1 | games,run | foo || bar")
    }

    #[test]
    fn string_parsing() {
        let daily_score_string = "2020-02-01 09:10:11 +0200 | 1 |  | foo || bar";
        let daily_score_parse_result = DailyScore::parse(daily_score_string);

        assert!(daily_score_parse_result.is_ok());

        let daily_score = daily_score_parse_result.unwrap();
        assert_eq!(daily_score.score, 1);
        assert_eq!(daily_score.comment, Some("foo || bar".to_string()));
        assert_eq!(daily_score.tags, HashSet::new());
        assert_eq!(Utc.ymd(2020, 2, 1).and_hms(7, 10, 11), daily_score.datetime);

        let daily_score_no_comment_string = "2020-02-01 09:10:11 +0200 | 1 | foo |";
        let daily_score_no_comment_parse_result = DailyScore::parse(daily_score_no_comment_string);

        assert!(daily_score_no_comment_parse_result.is_ok());

        let daily_score = daily_score_no_comment_parse_result.unwrap();
        assert_eq!(daily_score.comment, Some("".to_string()));
        assert_eq!(daily_score.tags.len(), 1);
        assert!(daily_score.tags.contains("foo"));
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
        assert_eq!(daily_score.tags, HashSet::new());
        assert_eq!(daily_score.comment, None);
    }

    #[test]
    fn new_with_score() {
        let daily_score = DailyScore::with_score(5);

        assert_eq!(daily_score.score, 5);
        assert_eq!(daily_score.tags, HashSet::new());
        assert_eq!(daily_score.comment, None);
    }

    #[test]
    fn errors_display() {
        assert_eq!(ParseError::MissingDateTime.to_string(), "datetime is missing");
        assert_eq!(ParseError::InvalidDateTime("foo bar".to_string()).to_string(), "'foo bar' is not a valid datetime");
        assert_eq!(ParseError::MissingScore.to_string(), "missing score");
        assert_eq!(ParseError::InvalidScore("foo".to_string()).to_string(), "'foo' is not a valid score");
    }
}
