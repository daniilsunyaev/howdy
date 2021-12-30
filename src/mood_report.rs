use chrono::{Local, Duration, Datelike, DateTime, FixedOffset};
use std::collections::{HashSet, HashMap};
use std::convert::TryFrom;

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
        let now = Local::now();
        let today = now.with_timezone(now.offset()).date();
        let last_monday = today.and_hms_nano(0, 0, 0, 0) - Duration::days(today.weekday().num_days_from_monday().into());
        self.iterative_const_period_report(last_monday, WEEK_SECONDS)
    }

    pub fn iterative_seven_days_mood(&self) -> Vec<(i64, i32)> {
        let now = Local::now();
        let today = now.with_timezone(now.offset()).date();
        let beginning_of_next_day = today.succ().and_hms_nano(0, 0, 0, 0);
        self.iterative_const_period_report(beginning_of_next_day, WEEK_SECONDS)
    }

    pub fn iterative_thirty_days_mood(&self) -> Vec<(i64, i32)> {
        let now = Local::now();
        let today = now.with_timezone(now.offset()).date();
        let beginning_of_next_day = today.succ().and_hms_nano(0, 0, 0, 0);
        self.iterative_const_period_report(beginning_of_next_day, DAY_SECONDS * 30)
    }

    pub fn iterative_monthly_mood(&self) -> Vec<(i64, i32)> {
        let now = Local::now();
        let now_fixed_offset = now.with_timezone(now.offset());
        let mut earliest_datetime = now_fixed_offset;
        let beginning_of_current_month = Self::beginning_of_month(now_fixed_offset);
        let mut monthly_scores: HashMap<i64, i32> = HashMap::new();

        for daily_score in self.daily_scores {
            if daily_score.datetime >= beginning_of_current_month { continue }
            if earliest_datetime > daily_score.datetime {
                earliest_datetime = daily_score.datetime;
            }

            // hash scores by 1st of corresponding score's month
            let monthly_score_sum = monthly_scores.entry(Self::beginning_of_month(daily_score.datetime).timestamp()).or_insert(0);
            *monthly_score_sum += daily_score.score as i32;
        }

        let mut data: Vec<(i64, i32)> = Vec::new();
        let mut beginning_of_month = beginning_of_current_month;
        while beginning_of_month > Self::beginning_of_month(earliest_datetime) {
            let beginning_of_previous_month = Self::beginning_of_previous_month(beginning_of_month);
            let previous_month_timestamp = beginning_of_previous_month.timestamp();
            // we need to get from hash by prev month 1st day timestamp (because it is easier to save it by
            // score's month 1st day timestamp), and store data as this month's 1st day timestamp
            data.push((beginning_of_month.timestamp(), *monthly_scores.get(&previous_month_timestamp).unwrap_or(&0)));
            beginning_of_month = beginning_of_previous_month;
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

    fn iterative_const_period_report(&self, report_ends_at: DateTime<FixedOffset>, period: i64) -> Vec<(i64, i32)> {
        let mut data = Vec::new();
        let filtered_daily_scores = self.daily_scores
            .iter()
            .filter(|daily_score| self.tags.iter().all(|tag| daily_score.tags.contains(tag)))
            .filter(|daily_score| daily_score.datetime < report_ends_at);

        for daily_score in filtered_daily_scores {
            let seconds_before_report_end = report_ends_at.timestamp() - daily_score.datetime.timestamp();
            // score at (report end - period duration) belongs to this period, so we have to subtract 1 second,
            // otherwise it will fall into previous period's report.
            let i_i64 = (seconds_before_report_end - 1) / period;

            // drop anything in the future or beyod platform's max len periods ago
            let i_convert = usize::try_from(i_i64);
            if i_convert.is_err() { continue };

            let i = i_convert.unwrap();
            let seconds_to_next_period = seconds_before_report_end % period;

            // potentially we can reserve data space before loop, but first record usually is the oldest,
            // so resize will happen only once in most of the cases
            if i >= data.len() {
                let mut len = data.len() as i64;
                data.resize_with(i + 1, || { len += 1; (report_ends_at.timestamp() - (len - 1) * period, 0) });
            }

            data[i] = (daily_score.datetime.timestamp() + seconds_to_next_period, data[i].1 + daily_score.score as i32);
        }
        data.reverse();
        data
    }

    fn beginning_of_month(datetime: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        datetime.date().and_hms_nano(0, 0, 0, 0) - Duration::days(datetime.day0().into())
    }

    fn beginning_of_previous_month(datetime: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        Self::beginning_of_month(datetime.date().pred().and_hms(0, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                datetime: last_monday() - Duration::days(15)
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
        let pre_previous_monday = last_monday() - Duration::days(14);

        assert_eq!(mood_report.iterative_weekly_mood(),
            vec![(pre_previous_monday.timestamp(), 4), (previous_monday.timestamp(), 0), (last_monday().timestamp(), 5)]
        )
    }

    #[test]
    fn iterative_seven_days_mood() {
        let daily_score = DailyScore::with_score(-10);
        let last_week_daily_score =
            DailyScore {
                score: 2,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: daily_score.datetime - Duration::days(1)
            };

        let another_last_week_daily_score =
            DailyScore {
                score: 3,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: daily_score.datetime - Duration::days(2)
            };

        let old_daily_score =
            DailyScore {
                score: 4,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: daily_score.datetime - Duration::days(15)
            };

        let mood_report =
            MoodReport {
                daily_scores: &vec![
                    another_last_week_daily_score,
                    old_daily_score,
                    last_week_daily_score,
                    daily_score,
                ],
                tags: &HashSet::new(),
            };

        assert_eq!(mood_report.iterative_seven_days_mood().len(), 3);

        let report_timestamp = mood_report.iterative_seven_days_mood()[2].0;
        let previous_period_end = report_timestamp - WEEK_SECONDS;
        let pre_previous_period_end = report_timestamp - 2 * WEEK_SECONDS;

        assert_eq!(mood_report.iterative_seven_days_mood(),
            vec![(pre_previous_period_end, 4), (previous_period_end, 0), (report_timestamp, -5)]
        )
    }

    #[test]
    fn iterative_thirty_days_mood() {
        let daily_score = DailyScore::with_score(-10);
        let last_month_daily_score =
            DailyScore {
                score: 3,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: daily_score.datetime - Duration::days(20)
            };

        let old_daily_score =
            DailyScore {
                score: 4,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: daily_score.datetime - Duration::days(61)
            };

        let mood_report =
            MoodReport {
                daily_scores: &vec![
                    old_daily_score,
                    last_month_daily_score,
                    daily_score,
                ],
                tags: &HashSet::new(),
            };

        assert_eq!(mood_report.iterative_thirty_days_mood().len(), 3);

        let report_timestamp = mood_report.iterative_thirty_days_mood()[2].0;
        let previous_period_end = report_timestamp - DAY_SECONDS * 30;
        let pre_previous_period_end = report_timestamp - 2 * DAY_SECONDS * 30;

        assert_eq!(mood_report.iterative_thirty_days_mood(),
            vec![(pre_previous_period_end, 4), (previous_period_end, 0), (report_timestamp, -7)]
        )
    }

    #[test]
    fn iterative_monthly_mood() {
        let daily_score = DailyScore::with_score(-10);
        let last_month_daily_score =
            DailyScore {
                score: 2,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: beginning_of_month() - Duration::days(1)
            };

        let another_last_month_daily_score =
            DailyScore {
                score: 3,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: beginning_of_month() - Duration::days(20)
            };

        let old_daily_score =
            DailyScore {
                score: 4,
                tags: HashSet::new(),
                comment: "".to_string(),
                datetime: beginning_of_month() - Duration::days(35)
            };

        let mood_report =
            MoodReport {
                daily_scores: &vec![
                    another_last_month_daily_score,
                    old_daily_score,
                    last_month_daily_score,
                    daily_score,
                ],
                tags: &HashSet::new(),
            };

        assert_eq!(mood_report.iterative_monthly_mood(),
            vec![(beginning_of_previous_month().timestamp(), 4), (beginning_of_month().timestamp(), 5)]
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

    fn beginning_of_month() -> DateTime<FixedOffset> {
        let today = now_with_fixed_offset().date();
        today.and_hms_nano(0, 0, 0, 0) - Duration::days(today.day0().into())
    }

    fn beginning_of_previous_month() -> DateTime<FixedOffset> {
        let last_day_of_previos_month = beginning_of_month() - Duration::days(1);
        last_day_of_previos_month - Duration::days(last_day_of_previos_month.day0().into())
    }
}
