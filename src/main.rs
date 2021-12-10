use std::env;
use std::process;

fn main() {
    let args = env::args();

    if let Err(message) = howdy::run(args) {
        eprintln!("execution failed:\n{}", message);
        process::exit(1)
    }
}
