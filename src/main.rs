#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    match stdin.read_line(&mut input) {
        Ok(_n) => println!("{}: command not found", input),
        Err(error) => println!("error: {error}"),
    }
}