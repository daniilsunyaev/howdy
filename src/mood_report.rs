use chrono::Local;
use chrono::Duration;
use std::collections::HashSet;

use crate::daily_score::DailyScore;

pub struct MoodReport {
    daily_scores: Vec<DailyScore>,
    tags: HashSet<String>,
}

impl MoodReport {
    #[cfg(test)]
    pub fn new() -> Self {
        Self { daily_scores: vec![], tags: HashSet::new() }
    }

    // TODO: do we need this method?
    pub fn from_daily_scores(daily_scores: Vec<DailyScore>) -> Self {
        Self { daily_scores, tags: HashSet::new() }
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
        let now = Local::now();
        let thirty_days_ago = (now - Duration::days(29)).with_timezone(now.offset());

        self.filter_mood_sum(|daily_score| daily_score.datetime.date() >= thirty_days_ago.date())
    }

    pub fn yearly_mood(&self) -> i32 {
        let now = Local::now();
        let usual_year_ago = (now - Duration::days(364)).with_timezone(now.offset());

        self.filter_mood_sum(|daily_score| daily_score.datetime.date() >= usual_year_ago.date())
    }

    pub fn thirty_days_moving_mood(&self) -> Vec<i32> {
        // from 29 days ago to now there are 30 calendar dates
        // also we use dates to verify if daily score records fit into frame
        // so 29-days frame covers 30 dates
        self.timeframed_moving_mood_report(29, 0, 29)
    }

    fn filter_mood_sum<F>(&self, filter_fn: F) -> i32
        where
            F: Fn(&&DailyScore) -> bool,
        {
            self.daily_scores
                .iter()
                .filter(filter_fn)
                .filter(|daily_score| self.tags.iter().all(|tag| daily_score.tags.contains(tag)))
                .map(|daily_score| daily_score.score as i32)
                .sum()
        }

    fn timeframed_moving_mood_report(&self, starts_at_days_ago: u32, ends_at_days_ago: u32, frame_size: u32) -> Vec<i32> {
        let mut hist = Vec::new();
        hist.reserve((starts_at_days_ago - ends_at_days_ago) as usize);
        let now = Local::now();
        let fixed_now = now.with_timezone(now.offset());

        for frame_ends_at_days_ago in (ends_at_days_ago..=starts_at_days_ago).rev() {
            let frame_end = fixed_now - Duration::days(frame_ends_at_days_ago as i64);
            let frame_start = frame_end - Duration::days(frame_size as i64);
            let sum = self
                .filter_mood_sum(|daily_score| {
                    daily_score.datetime.date() >= frame_start.date() && daily_score.datetime.date() <= frame_end.date()
                });
            hist.push(sum);
        }

        hist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, FixedOffset, DateTime};

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
            DailyScore {
                score: 5,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(40)
            };

        let mood_report =
            MoodReport::from_daily_scores(vec![daily_score, another_daily_score, old_daily_score]);

        assert_eq!(mood_report.thirty_days_mood(), 3);
    }

    #[test]
    fn thirty_days_moving_mood() {
        let today_daily_score = DailyScore::with_score(1);
        let beginning_of_month_daily_score =
            DailyScore {
                score: -1,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(25) - Duration::minutes(1)
            };
        let fifty_days_ago_daily_score =
            DailyScore {
                score: 2,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(50) + Duration::minutes(1)
            };
        let ninty_days_ago_daily_score =
            DailyScore {
                score: 20,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(90)
            };

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
                2, 2, 2, 2, 1, 1, 1, 1, 1, -1,
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
            DailyScore {
                score: 5,
                datetime: now_with_fixed_offset()  - Duration::days(40),
                tags: HashSet::new(),
                comment: "".to_string(),
            };

        let old_score =
            DailyScore {
                score: -4,
                datetime: now_with_fixed_offset() - Duration::weeks(55),
                tags: vec!["tag".to_string()].into_iter().collect(),
                comment: "".to_string(),
            };

        let mood_report = MoodReport::from_daily_scores(
            vec![daily_score, another_daily_score, forty_days_ago_score, old_score]
        );

        assert_eq!(mood_report.yearly_mood(), 8);
    }

    fn now_with_fixed_offset() -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&FixedOffset::east(0))
    }
}
