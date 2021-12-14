use std::env;
use std::process;
use std::error::Error;

fn main() {
    let args = env::args();

    if let Err(error) = howdy::run(args) {
        report_error(&error);
        process::exit(1)
    }
}

fn report_error(error: &dyn Error) {
    eprintln!("execution failed:");
    let mut current = error;
    loop {
        eprintln!("\t{}", current);
        if current.source().is_none() {
            break;
        } else {
            current = current.source().unwrap();
        }
    }
}
