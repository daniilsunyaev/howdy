use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use crate::daily_score::DailyScore;

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const JOURNAL_SEPARATOR: char = '|';

mod daily_score;

pub struct InputArgs {
    score: i8,
    comment: String,
}


impl InputArgs {
    pub fn parse<I>(mut args: I) -> Result<InputArgs, &'static str>
    where
        I: Iterator<Item = String>,
    {
        args.next(); // skip exec filename
        let command = match args.next() {
            Some(arg) => arg,
            None => return Err("command is not provided"),
        };

        match command.as_str() {
            "add" => return InputArgs::parse_add_args(args),
            _ => return Err("command is not recognized"),
        }
    }

    fn parse_add_args<I>(mut args: I) -> Result<InputArgs, &'static str>
    where
        I: Iterator<Item = String>,
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_args(args_str: &str) -> std::vec::IntoIter<String> {
        args_str.split(' ').map(|s| s.to_string()).collect::<Vec<String>>().into_iter()
    }

    #[test]
    fn no_command_error() {
        let args = Vec::new();

        assert_eq!(InputArgs::parse(args.into_iter()).err(), Some("command is not provided"));
    }

    #[test]
    fn wrong_command_error() {
        let args = build_args("exec/path foo");

        assert_eq!(InputArgs::parse(args).err(), Some("command is not recognized"));
    }

    #[test]
    fn no_add_args_error() {
        let args = build_args("exec/path add");

        assert_eq!(InputArgs::parse(args.into_iter()).err(), Some("failed to get daily score"));
    }

    #[test]
    fn wrong_score_error() {
        let args = build_args("exec/path add x");

        assert_eq!(InputArgs::parse(args.into_iter()).err(), Some("failed to parse daily score"));
    }

    #[test]
    fn correct_args() {
        let args = build_args("exec/path add -1");
        let parsed = InputArgs::parse(args);

        assert_eq!(parsed.is_ok(), true);


        let parsed = parsed.unwrap();

        assert_eq!(parsed.score, -1);
        assert_eq!(parsed.comment, "");
    }
}
