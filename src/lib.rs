use crate::add_command::AddCommand;
use crate::mood_command::MoodCommand;

const JOURNAL_FILE_PATH: &str = "./howdy.journal";
const JOURNAL_SEPARATOR: char = '|';

mod daily_score;
mod add_command;
mod mood_command;
mod mood_report;

pub fn run<I>(mut cli_args: I) -> Result<(), &'static str>
    where
        I: Iterator<Item = String>,
    {
    cli_args.next(); // skip exec filename
    let command = match cli_args.next() {
        Some(arg) => arg,
        None => return Err("command is not provided"),
    };

    match command.as_str() {
        "add" => AddCommand::parse(cli_args)?.run(),
        "mood" => (MoodCommand {}).run(),
        _ => return Err("command is not recognized"),
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_args(args_str: &str) -> impl Iterator<Item = String> + '_ {
        args_str.split(' ').map(|s| s.to_string())
    }

    #[test]
    fn no_command_error() {
        let args = Vec::new();

        assert_eq!(run(args.into_iter()).err(), Some("command is not provided"));
    }

    #[test]
    fn wrong_command_error() {
        let args = build_args("exec/path foo");

        assert_eq!(run(args).err(), Some("command is not recognized"));
    }

    #[test]
    fn no_add_args_error() {
        let args = build_args("exec/path add");

        assert_eq!(run(args.into_iter()).err(), Some("failed to get daily score"));
    }
}
