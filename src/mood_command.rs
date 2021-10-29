use std::io::prelude::*; // TODO: get rid of glob imports
use std::io::BufReader;
use std::fs::OpenOptions;

use crate::daily_score::DailyScore;
use crate::mood_report::MoodReport;

pub struct MoodCommand {
}

impl MoodCommand {
    pub fn run(&self) -> Result<(), &'static str> {
        let mut records = Vec::<DailyScore>::new();
        let open_journal = OpenOptions::new()
            .read(true)
            .open(crate::JOURNAL_FILE_PATH);

        match open_journal {
            Err(message) => println!("error opening journal file: {}", message),
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    match line {
                        Err(_message) => return Err("error reading the line"),
                        Ok(line) => {
                            if let Ok(daily_score) = DailyScore::parse(&line) {
                                records.push(daily_score);
                            } else {
                                return Err("error parsing the line");
                            }
                        }
                    }
                }
            }
        }

        let mood_report = MoodReport::from_daily_scores(records);

        println!("30-days mood: {}", mood_report.thirty_days_mood());
        println!("365-days mood: {}", mood_report.yearly_mood());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_ok() {
        let mood = MoodCommand {};

        assert_eq!(mood.run().is_err(), false);
    }
}
