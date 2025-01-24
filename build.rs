use std::process;
use std::process::Command;

fn main() {
    process::exit(Command::new("make").status().unwrap().code().unwrap());
}
