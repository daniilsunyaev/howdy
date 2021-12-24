use chrono::Local;
use chrono::Duration;
use std::collections::HashSet;

use crate::daily_score::DailyScore;

pub struct MoodReport<'a> {
    pub daily_scores: &'a Vec<DailyScore>,
    pub tags: &'a HashSet<String>,
}

impl<'a> MoodReport<'a> {
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

    pub fn thirty_days_moving_mood(&self) -> Vec<(i64, i32)> {
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

    fn timeframed_moving_mood_report(&self, starts_at_days_ago: u32, ends_at_days_ago: u32, frame_size: u32) -> Vec<(i64, i32)> {
        let mut hist = Vec::new();
        hist.reserve((starts_at_days_ago - ends_at_days_ago) as usize);
        let now = Local::now();
        let fixed_now = now.with_timezone(now.offset());

        for frame_ends_at_days_ago in (ends_at_days_ago..=starts_at_days_ago).rev() {
            let frame_ends_at_timestamp = Local::now().timestamp() - (frame_ends_at_days_ago as i64 * 3600 * 24);
            let frame_end = fixed_now - Duration::days(frame_ends_at_days_ago as i64);
            let frame_start = frame_end - Duration::days(frame_size as i64);
            let sum = self
                .filter_mood_sum(|daily_score| {
                    daily_score.datetime.date() >= frame_start.date() && daily_score.datetime.date() <= frame_end.date()
                });
            hist.push((frame_ends_at_timestamp, sum));
        }

        hist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, FixedOffset, DateTime};

    #[test]
    fn consumes_scores() {
        let scores = vec![DailyScore::new(), DailyScore::new()];
        let mood_report = MoodReport { daily_scores: &scores, tags: &HashSet::new() };

        assert_eq!(mood_report.len(), 2);
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
            MoodReport {
                daily_scores: &vec![daily_score, another_daily_score, old_daily_score],
                tags: &HashSet::new(),
            };


        assert_eq!(mood_report.thirty_days_mood(), 3);
    }

    #[test]
    fn thirty_days_mood_with_tags() {
        let tag: HashSet<String> = vec!["tag".to_string()].into_iter().collect();
        let tag2: HashSet<String> = vec!["tag2".to_string()].into_iter().collect();

        let daily_score =
            DailyScore {
                score: 1,
                tags: tag2.clone(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset(),
            };

        let another_daily_score =
            DailyScore {
                score: 2,
                tags: tag.clone(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(20)
            };

        let old_daily_score =
            DailyScore {
                score: 5,
                tags: tag.clone(),
                comment: "".to_string(),
                datetime: now_with_fixed_offset() - Duration::days(40)
            };

        let daily_scores = vec![daily_score, another_daily_score, old_daily_score];

        let tag_mood_report =
            MoodReport {
                daily_scores: &daily_scores,
                tags: &tag,
            };

        let multitag_mood_report =
            MoodReport {
                daily_scores: &daily_scores,
                tags: &vec!["tag".to_string(), "tag2".to_string()].into_iter().collect(),
            };

        assert_eq!(tag_mood_report.thirty_days_mood(), 2);
        assert_eq!(multitag_mood_report.thirty_days_mood(), 0);
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

        let mood_report = MoodReport {
            daily_scores: &vec![
                    beginning_of_month_daily_score,
                    fifty_days_ago_daily_score,
                    ninty_days_ago_daily_score,
                    today_daily_score
                ],
            tags: &HashSet::new(),
        };

        assert_eq!(mood_report.thirty_days_moving_mood().iter().map(|val| val.1).collect::<Vec<i32>>(),
            vec![
                2, 2, 2, 2, 1, 1, 1, 1, 1, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, -1, -1, -1, -1, -1, 0,
            ]
        );
    }

    #[test]
    fn yearly_mood() {
        let no_tags = HashSet::new();
        let tag_tags: HashSet<String> = vec!["tag".to_string()].into_iter().collect();

        let daily_score = DailyScore::with_score(1);
        let another_daily_score = DailyScore::with_score(2);
        let forty_days_ago_score =
            DailyScore {
                score: 5,
                datetime: now_with_fixed_offset()  - Duration::days(40),
                tags: no_tags.clone(),
                comment: "".to_string(),
            };

        let old_score =
            DailyScore {
                score: -4,
                datetime: now_with_fixed_offset() - Duration::weeks(55),
                tags: tag_tags.clone(),
                comment: "".to_string(),
            };

        let daily_scores = vec![daily_score, another_daily_score, forty_days_ago_score, old_score];

        let mood_report = MoodReport { daily_scores: &daily_scores, tags: &no_tags };
        let tagged_mood_report = MoodReport { daily_scores: &daily_scores, tags: &tag_tags };

        assert_eq!(mood_report.yearly_mood(), 8);
        assert_eq!(tagged_mood_report.yearly_mood(), 0);
    }

    fn now_with_fixed_offset() -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&FixedOffset::east(0))
    }
}
