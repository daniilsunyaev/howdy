use chrono::prelude::{DateTime, Utc};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub struct DailyScore {
    pub score: i8,
    pub comment: String,
    pub datetime: DateTime<Utc>,
}

impl DailyScore {
    #[cfg(test)]
    pub fn new() -> Self {
        Self { score: 0, comment: "".to_string(), datetime: Utc::now() }
    }

    #[cfg(test)]
    pub fn with_score(score: i8) -> Self {
        Self { score, ..Self::new() }
    }

    pub fn to_s(&self) -> String {
        format!("{} {} {} {} {}", self.datetime.format(DATE_FORMAT), crate::JOURNAL_SEPARATOR, self.score, crate::JOURNAL_SEPARATOR, self.comment)
    }

    pub fn parse(daily_score_string: &str) -> Result<Self, &str> { // TODO: prob should use box dyn error or something
        let mut slice = daily_score_string.splitn(3, " | "); // TODO: use separator instead

        let datetime_str = slice.next().unwrap_or("");
        if let Ok(datetime) = DateTime::parse_from_str(datetime_str, DATE_FORMAT) {
            let score_str = slice.next().unwrap_or("");

            if let Ok(score) = score_str.parse::<i8>() {
                let comment = slice.next().unwrap_or("").to_string();
                let datetime: DateTime<Utc> = DateTime::from(datetime);
                Ok(DailyScore { score, comment, datetime })
            } else {
                Err("failed to parse score")
            }
        } else {
            Err("failed to parse datetime")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_formatting() {
        let score = DailyScore { score: 1, comment: "foo || bar".to_string(), datetime: Utc.ymd(2020, 1, 1).and_hms(9, 10, 11) };

        assert_eq!(score.to_s(), "2020-01-01 09:10:11 +0000 | 1 | foo || bar")
    }

    #[test]
    fn string_parsing() {
        let daily_score_string = "2020-02-01 09:10:11 +0000 | 1 | foo || bar";
        let daily_score_parse_result = DailyScore::parse(daily_score_string);

        assert!(daily_score_parse_result.is_ok());

        let daily_score = daily_score_parse_result.unwrap();
        assert_eq!(daily_score.score, 1);
        assert_eq!(daily_score.comment, "foo || bar");
        assert_eq!(Utc.ymd(2020, 2, 1).and_hms(9, 10, 11), daily_score.datetime);
    }

    #[test]
    fn new_defaults() {
        let daily_score = DailyScore::new();

        assert_eq!(daily_score.score, 0);
        assert_eq!(daily_score.comment, "");
    }

    #[test]
    fn new_with_score() {
        let daily_score = DailyScore::with_score(5);

        assert_eq!(daily_score.score, 5);
        assert_eq!(daily_score.comment, "");
    }
}
