use chrono::{Local, Duration, Datelike};
use std::collections::HashSet;

use crate::daily_score::DailyScore;

const HOUR_SECONDS: i64 = 3600;
const DAY_SECONDS: i64 = HOUR_SECONDS * 24;
const WEEK_SECONDS: i64 = DAY_SECONDS * 7;

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

    pub fn iterative_weekly_mood(&self) -> Vec<(i64, i32)> {
        let today = Local::now().date();
        let last_monday = today.and_hms_nano(0, 0, 0, 0) - Duration::days(today.weekday().num_days_from_monday().into());
        let mut data: Vec<(i64, i32)> = Vec::new();

        for daily_score in self.daily_scores {
            if daily_score.datetime >= last_monday { continue }
            let seconds_before_last_monday = last_monday.timestamp() - daily_score.datetime.timestamp();
            // Mon 00:00:00 belongs to the this week and to the next Monday's report,
            // so we have to substract 1 second. Otherwise it will fall into previous week's report.
            // Due to previous checks this value is always > 0 before sutraction, `as usize` is safe
            let i = ((seconds_before_last_monday - 1) / WEEK_SECONDS) as usize;
            let seconds_to_succ_monday = seconds_before_last_monday % WEEK_SECONDS;

            // potentially we can reserve data space before loop, but first record usually is the oldest,
            // so resize will happen only once in most of the cases
            if i >= data.len() {
                let mut len = data.len() as i64;
                data.resize_with(i + 1, || { len += 1; (last_monday.timestamp() - len * WEEK_SECONDS, 0)});
            }
            data[i] = (daily_score.datetime.timestamp() + seconds_to_succ_monday, data[i].1 + daily_score.score as i32);
        }
        data.reverse();

        data
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
            let frame_ends_at_timestamp = Local::now().timestamp() - (frame_ends_at_days_ago as i64 * DAY_SECONDS);
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
    use chrono::{FixedOffset, DateTime};

    #[test]
    fn consumes_scores() {
        let scores = vec![DailyScore::new(), DailyScore::new()];
        let mood_report = MoodReport { daily_scores: &scores, tags: &HashSet::new() };

        assert_eq!(mood_report.len(), 2);
    }

    #[test]
    fn iterative_weekly_mood() {
        let daily_score = DailyScore::with_score(-10);
        let last_week_daily_score =
            DailyScore {
                score: 2,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: last_monday() - Duration::days(1)
            };

        let another_last_week_daily_score =
            DailyScore {
                score: 3,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: last_monday() - Duration::days(2)
            };

        let old_daily_score =
            DailyScore {
                score: 4,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: last_monday() - Duration::days(8)
            };

        let mood_report =
            MoodReport {
                daily_scores: &vec![
                    another_last_week_daily_score,
                    daily_score,
                    old_daily_score,
                    last_week_daily_score,
                ],
                tags: &HashSet::new(),
            };

        let previous_monday = last_monday() - Duration::days(7);

        assert_eq!(mood_report.iterative_weekly_mood(),
            vec![(previous_monday.timestamp(), 4), (last_monday().timestamp(), 5)]
        )
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
        let now = Local::now();
        now.with_timezone(now.offset())
    }

    fn last_monday() -> DateTime<FixedOffset> {
        let today = now_with_fixed_offset().date();
        today.and_hms_nano(0, 0, 0, 0) - Duration::days(today.weekday().num_days_from_monday().into())
    }
}
