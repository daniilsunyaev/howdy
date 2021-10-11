use std::env;
use std::process;
use howdy::InputArgs;

fn main() {
    let input_args = InputArgs::parse(env::args()).unwrap_or_else(|err| {
        eprintln!("error parsing arguments: {}", err);
        process::exit(1);
    });

    howdy::run(input_args);
}
