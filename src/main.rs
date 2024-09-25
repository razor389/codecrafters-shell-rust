#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, process::{self, Command}};

fn main() {
    let stdin = io::stdin();

    // List of built-in commands
    let builtins = vec!["echo", "exit", "type"];

    loop {
        // Print the shell prompt
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
        } 
        // Check if the command is 'pwd'
        else if command == "pwd" {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("Error getting current directory: {}", e),
            }
        }
        // Check if the command starts with 'type'
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
        } 
         // Try to run the command as an executable with arguments
         else if !command.is_empty() {
            let mut parts = command.split_whitespace();
            if let Some(executable) = parts.next() {
                let args: Vec<&str> = parts.collect();
                
                if let Some(executable_path) = find_in_path(executable) {
                    // Execute the command with arguments
                    match run_command(&executable_path, &args) {
                        Ok(output) => print!("{}", output),
                        Err(err) => eprintln!("Error: {}", err),
                    }
                } else {
                    println!("{}: command not found", executable);
                }
            }
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

// Function to run a command with arguments
fn run_command(executable: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(executable)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", executable, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}