use chrono::Utc;
use chrono::Duration;

use crate::daily_score::DailyScore;

pub struct MoodReport {
    daily_scores: Vec<DailyScore>,
}

impl MoodReport {
    #[cfg(test)]
    pub fn new() -> Self {
        Self { daily_scores: vec![] }
    }

    pub fn from_daily_scores(daily_scores: Vec<DailyScore>) -> Self {
        Self { daily_scores }
    }

    pub fn add_score(&mut self, daily_score: DailyScore) -> &Self {
        self.daily_scores.push(daily_score);
        self
    }

    pub fn len(&self) -> usize {
        self.daily_scores.len()
    }

    pub fn thirty_days_mood(&self) -> i32 {
        let thirty_days_ago = Utc::now() - Duration::days(40);

        self.filter_mood_sum(|daily_score| daily_score.datetime >= thirty_days_ago)
    }

    pub fn yearly_mood(&self) -> i32 {
        let usual_year_ago = Utc::now() - Duration::days(365);

        self.filter_mood_sum(|daily_score| daily_score.datetime >= usual_year_ago)
    }

    fn filter_mood_sum<F>(&self, filter_fn: F) -> i32
        where
            F: Fn(&&DailyScore) -> bool,
        {
            self.daily_scores
                .iter()
                .filter(filter_fn)
                .map(|daily_score| daily_score.score as i32)
                .sum()
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_default() {
        let mood = MoodReport::new();

        assert_eq!(mood.len(), 0);
    }

    #[test]
    fn consumes_scores() {
        let scores = vec![DailyScore::new(), DailyScore::new()];
        let mood_report = MoodReport::from_daily_scores(scores);

        assert_eq!(mood_report.len(), 2);
    }

    #[test]
    fn adds_score() {
        let mut mood = MoodReport::new();
        let daily_score = DailyScore::new();
        let another_daily_score = DailyScore::new();

        assert_eq!(mood.add_score(daily_score).len(), 1);
        assert_eq!(mood.add_score(another_daily_score).len(), 2);
    }

    #[test]
    fn thirty_days_mood() {
        let daily_score = DailyScore::with_score(1);
        let another_daily_score = DailyScore::with_score(2);
        let old_daily_score =
            DailyScore { score: 5, comment: "".to_string(), datetime: Utc::now() - Duration::days(40) };

        let mood_report =
            MoodReport::from_daily_scores(vec![daily_score, another_daily_score, old_daily_score]);

        assert_eq!(mood_report.thirty_days_mood(), 3);
    }

    #[test]
    fn yearly_mood() {
        let daily_score = DailyScore::with_score(1);
        let another_daily_score = DailyScore::with_score(2);
        let forty_days_ago_score =
            DailyScore { score: 5, comment: "".to_string(), datetime: Utc::now() - Duration::days(40) };

        let old_score =
            DailyScore { score: -4, datetime: Utc::now() - Duration::weeks(55), comment: "".to_string() };

        let mood_report = MoodReport::from_daily_scores(
            vec![daily_score, another_daily_score, forty_days_ago_score, old_score]
        );

        assert_eq!(mood_report.yearly_mood(), 8);
    }
}
