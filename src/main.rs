use std::env;
use std::process;

fn main() {
    let args = env::args();

    if let Err(message) = howdy::run(args) {
        eprintln!("error: {}", message);
        process::exit(1)
    }
}
