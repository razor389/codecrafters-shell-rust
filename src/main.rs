#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    let stdin = io::stdin();

    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read the user's input
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // Trim the input to remove newline and extra spaces
        let command = input.trim();

        // Check if the command is 'exit 0'
        if command == "exit 0" {
            process::exit(0); // Exit with status 0
        }

        // Check if the command starts with 'echo'
        if command.starts_with("echo ") {
            // Extract the part after 'echo ' and print it
            let echo_message = &command[5..]; // Get everything after 'echo '
            println!("{}", echo_message);
        } else if !command.is_empty() {
            // If command is not empty and isn't 'echo', show the 'command not found' message
            println!("{}: command not found", command);
        }
    }
}
