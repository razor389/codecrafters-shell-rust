#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Prompt the user for input
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    // Trim the input to remove any trailing newline or spaces
    let command = input.trim();

    // Print the '<command>: command not found' message if a command is entered
    if !command.is_empty() {
        println!("{}: command not found", command);
    }
}
