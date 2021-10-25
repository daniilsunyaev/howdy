use std::env;
use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const JOURNAL_SEPARATOR: char = '|';

struct DailyScore {
    score: i8,
    comment: String,
    datetime: DateTime<Utc>,
}

pub struct InputArgs {
    score: i8,
    comment: String,
}

impl DailyScore {
    pub fn to_s(&self) -> String {
        format!("{} {} {} {} {}", self.datetime, JOURNAL_SEPARATOR, self.score, JOURNAL_SEPARATOR, self.comment)
    }
}

impl InputArgs {
    pub fn parse(mut args: env::Args) -> Result<InputArgs, &'static str> {
        args.next(); // skip exec filename
        let score_string = match args.next() {
            Some(arg) => arg,
            None => return Err("failed to get daily score"),
        };
        let score: i8 = match score_string.parse() {
            Ok(int_score) => int_score,
            Err(_message) => return Err("failed to parse daily score"),
        };

        let comment = args.next().unwrap_or("".to_string());
        let stuff = args.next().unwrap_or("".to_string());
        println!("stuff is {}", stuff);

        return Ok(InputArgs { score, comment })
    }
}

pub fn run(input_args: InputArgs) {
    let today = DailyScore {
        score: input_args.score,
        comment: input_args.comment,
        datetime: Utc::now(),
    };

    println!(
        "today's score is {}, with comment \"{}\", and its time {}",
        today.score, today.comment, today.datetime
    );

    let open_journal = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(JOURNAL_FILE_PATH);

    match open_journal {
        Err(message) => println!("error opening or creating a file: {}", message),
        Ok(mut file) => {
            println!("opened file successfully!");
            if let Err(message) = writeln!(file, "{}", today.to_s()) {
                println!("error appending to a file: {}", message);
            }
        }
    }
}
