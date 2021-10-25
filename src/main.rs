use std::env;
use std::process;
use howdy::InputArgs;

fn main() {
    let args = env::args();

    let input_args = InputArgs::parse(args).unwrap_or_else(|err| {
        eprintln!("error parsing arguments: {}", err);
        process::exit(1);
    });

    howdy::run(input_args);
}
