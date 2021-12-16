use chrono::Local;
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

    #[cfg(test)]
    pub fn add_score(&mut self, daily_score: DailyScore) -> &Self {
        self.daily_scores.push(daily_score);
        self
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.daily_scores.len()
    }

    pub fn thirty_days_mood(&self) -> i32 {
        let thirty_days_ago = Local::now() - Duration::days(30);

        self.filter_mood_sum(|daily_score| daily_score.datetime >= thirty_days_ago)
    }

    pub fn yearly_mood(&self) -> i32 {
        let usual_year_ago = Local::now() - Duration::days(365);

        self.filter_mood_sum(|daily_score| daily_score.datetime >= usual_year_ago)
    }

    pub fn thirty_days_moving_mood(&self) -> Vec<i32> {
        self.timeframed_moving_mood_report(30, 0, 30)
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

    fn timeframed_moving_mood_report(&self, starts_at_days_ago: u32, ends_at_days_ago: u32, frame_size: u32) -> Vec<i32> {
        let mut hist = Vec::new();
        hist.reserve((starts_at_days_ago - ends_at_days_ago) as usize);

        for frame_starts_at_days_ago in ((ends_at_days_ago + frame_size)..(starts_at_days_ago + frame_size)).rev() {
            let sum = self
                .filter_mood_sum(|daily_score| {
                    daily_score.datetime >= Local::now() - Duration::days(frame_starts_at_days_ago as i64) &&
                        daily_score.datetime <= Local::now() - Duration::days(frame_starts_at_days_ago as i64) + Duration::days(frame_size as i64)
                });
            hist.push(sum);
        }

        hist
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
            DailyScore { score: 5, comment: "".to_string(), datetime: Local::now() - Duration::days(40) };

        let mood_report =
            MoodReport::from_daily_scores(vec![daily_score, another_daily_score, old_daily_score]);

        assert_eq!(mood_report.thirty_days_mood(), 3);
    }

    #[test]
    fn thirty_days_moving_mood() {
        let today_daily_score = DailyScore::with_score(1);
        let beginning_of_month_daily_score =
            DailyScore { score: -1, comment: "".to_string(), datetime: Local::now() - Duration::days(25) - Duration::minutes(1) };
        let fifty_days_ago_daily_score =
            DailyScore { score: 2, comment: "".to_string(), datetime: Local::now() - Duration::days(50) + Duration::minutes(1) };
        let ninty_days_ago_daily_score =
            DailyScore { score: 20, comment: "".to_string(), datetime: Local::now() - Duration::days(90) };

        let mood_report = MoodReport::from_daily_scores(
            vec![
                beginning_of_month_daily_score,
                fifty_days_ago_daily_score,
                ninty_days_ago_daily_score,
                today_daily_score
            ]
        );

        assert_eq!(mood_report.thirty_days_moving_mood(),
            vec![
                2, 2, 2, 2, 1, 1, 1, 1, 1, 1,
                -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, -1, 0,
            ]
        );
    }

    #[test]
    fn yearly_mood() {
        let daily_score = DailyScore::with_score(1);
        let another_daily_score = DailyScore::with_score(2);
        let forty_days_ago_score =
            DailyScore { score: 5, comment: "".to_string(), datetime: Local::now() - Duration::days(40) };

        let old_score =
            DailyScore { score: -4, datetime: Local::now() - Duration::weeks(55), comment: "".to_string() };

        let mood_report = MoodReport::from_daily_scores(
            vec![daily_score, another_daily_score, forty_days_ago_score, old_score]
        );

        assert_eq!(mood_report.yearly_mood(), 8);
    }
}
