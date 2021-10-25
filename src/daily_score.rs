use chrono::prelude::*;

pub struct DailyScore {
    pub score: i8,
    pub comment: String,
    pub datetime: DateTime<Utc>,
}

impl DailyScore {
    pub fn to_s(&self) -> String {
        format!("{} {} {} {} {}", self.datetime, crate::JOURNAL_SEPARATOR, self.score, crate::JOURNAL_SEPARATOR, self.comment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_formatting() {
        let score = DailyScore { score: 1, comment: "foo || bar".to_string(), datetime: Utc.ymd(2020, 1, 1).and_hms(9, 10, 11) };

        assert_eq!(score.to_s(), "2020-01-01 09:10:11 UTC | 1 | foo || bar")
    }
}
