#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, process::{self, Command}};

fn main() {
    let stdin = io::stdin();

    // List of built-in commands
    let builtins = vec!["echo", "exit", "type", "pwd", "cd"];

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
            // Extract the part after 'echo '
            let echo_message = &command[5..];
        
            if echo_message.starts_with('\'') && echo_message.ends_with('\'') {
                // Handle single quotes: treat as literal
                let trimmed_message = &echo_message[1..echo_message.len() - 1];
                println!("{}", trimmed_message);
            } else if echo_message.starts_with('"') && echo_message.ends_with('"') {
                // Handle double quotes: interpret special characters
                let trimmed_message = &echo_message[1..echo_message.len() - 1];
                let interpreted_message = interpret_special_characters(trimmed_message);
                println!("{}", interpreted_message);
            } else {
                // Normalize spaces: Split, trim, and join
                let normalized_message = echo_message
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ");
                println!("{}", normalized_message);
            }

            // Explicitly print the prompt for the next command
            continue;
        }

        // Handle the 'cd' command
        else if command.starts_with("cd ") {
            let target_dir = &command[3..]; // Extract the directory path after 'cd '
            // Handle 'cd ~' by changing to the home directory
            let dir_to_change = if target_dir == "~" {
                // Get the home directory from the HOME environment variable
                env::var("HOME").unwrap_or_else(|_| String::from("/"))
            } else {
                target_dir.to_string()
            };
            if let Err(_e) = env::set_current_dir(&dir_to_change) {
                eprintln!("cd: {}: No such file or directory", dir_to_change);
            }
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
        // Check if the command starts with 'cat'
        if command.starts_with("cat ") {
            // Extract the file paths after 'cat '
            let file_paths = &command[4..];
            // Split the file paths and process each
            for file_path in file_paths.split_whitespace() {
                match fs::read_to_string(file_path) {
                    Ok(content) => print!("{}", content),
                    Err(err) => eprintln!("cat: {}: {}", file_path, err),
                }
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

// Function to interpret special characters within double quotes
fn interpret_special_characters(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Handle escaped characters
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        _ => result.push(escaped),
                    }
                }
            }
            '$' => {
                // Handle environment variables
                let mut var_name = String::new();
                while let Some(&next_char) = chars.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        var_name.push(next_char);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(value) = env::var(&var_name) {
                    result.push_str(&value);
                }
            }
            _ => result.push(c),
        }
    }

    result
}
      