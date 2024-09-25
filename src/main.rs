#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, process};

fn main() {
    let stdin = io::stdin();
    // List of built-in commands
    let builtins = vec!["echo", "exit", "type"];

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
        } // Check if the command starts with 'type'
        else if command.starts_with("type ") {
            let target_command = &command[5..]; // Get the command after 'type '
            
            // Check if the target command is a built-in command
            if builtins.contains(&target_command) {
                println!("{} is a shell builtin", target_command);
            } 
            // Check if the target command is an executable in the PATH
            else if let Some(executable_path) = find_in_path(target_command) {
                println!("{} is {}", target_command, executable_path);
            }
            else {
                println!("{}: not found", target_command);
            }
        } else if !command.is_empty() {
            // If command is not empty and isn't 'echo', show the 'command not found' message
            println!("{}: command not found", command);
        }
    }
}

// Function to search for an executable in the system's PATH
fn find_in_path(executable: &str) -> Option<String> {
    if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(':') {
            let full_path = format!("{}/{}", path, executable);

            // Check if the file exists and is executable
            if fs::metadata(&full_path).is_ok() {
                return Some(full_path);
            }
        }
    }
    None
}