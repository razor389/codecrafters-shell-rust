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
        
            // Use a vector to store segments
            let mut segments = Vec::new();
            let mut current_segment = String::new();
            let mut in_quotes = false;
            let mut quote_char = '\0';
        
            let mut chars = echo_message.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '\'' | '"' => {
                        if in_quotes && c == quote_char {
                            // End of quoted segment
                            in_quotes = false;
                            if quote_char == '"' {
                                let interpreted_segment = interpret_special_characters(&current_segment);
                                segments.push(interpreted_segment);
                            } else {
                                segments.push(current_segment.clone());
                            }
                            current_segment.clear();
                        } else if !in_quotes {
                            // Start of quoted segment
                            in_quotes = true;
                            quote_char = c;
                        } else {
                            // Inside quotes, include the quote character
                            current_segment.push(c);
                        }
                    }
                    '\\' => {
                        if in_quotes {
                            if quote_char == '"' {
                                // Inside double quotes, backslash escapes certain characters
                                if let Some(next_char) = chars.next() {
                                    current_segment.push('\\');
                                    current_segment.push(next_char);
                                } else {
                                    // Trailing backslash inside quotes
                                    current_segment.push('\\');
                                }
                            } else {
                                // Inside single quotes, backslash is treated literally
                                current_segment.push('\\');
                            }
                        } else {
                            // Outside quotes, backslash escapes the next character
                            if let Some(next_char) = chars.next() {
                                current_segment.push(next_char);
                            } else {
                                // Trailing backslash outside quotes
                                current_segment.push('\\');
                            }
                        }
                    }
                    ' ' if !in_quotes => {
                        // Space outside quotes indicates separation between words
                        if !current_segment.is_empty() {
                            segments.push(current_segment.clone());
                            current_segment.clear();
                        }
                        // If current_segment is empty, we skip adding to segments
                        // This effectively normalizes multiple spaces to a single separator
                    }
                    _ => {
                        current_segment.push(c);
                    }
                }
            }
        
            // Add any remaining segment
            if !current_segment.is_empty() {
                segments.push(current_segment);
            }
        
            // Join segments with spaces and print
            println!("{}", segments.join(" "));
        
            // Continue to the next command
            continue;
        }                      
                   
        // Handle the 'cd' command
        if command.starts_with("cd ") || command == "cd" {
            let args = if command.len() > 2 { &command[3..].trim() } else { "" }; // Extract the part after 'cd', or empty for just 'cd'

            // Determine the target directory
            let target_dir = if args.is_empty() || args == "~" {
                env::var("HOME").unwrap_or_else(|_| String::from("/"))
            } else {
                args.to_string()
            };

            // Attempt to change the directory
            if let Err(_) = env::set_current_dir(&target_dir) {
                eprintln!("cd: {}: No such file or directory", target_dir);
            }

            // Explicitly continue to the next command
            continue;
        }

        // Check if the command is 'pwd'
        else if command == "pwd" {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("Error getting current directory: {}", e),
            }

            continue;
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

            continue;
        } 
        // Check if the command starts with 'cat'
        if command.starts_with("cat ") {
            // Extract the arguments after 'cat '
            let args = &command[4..];
        
            // Parse arguments to handle quoted paths
            let mut file_paths = vec![];
            let mut current_path = String::new();
            let mut in_quotes = false;
            let mut quote_char = '\0';
        
            for c in args.chars() {
                match c {
                    '\'' | '"' if !in_quotes => {
                        // Start a quoted path
                        in_quotes = true;
                        quote_char = c;
                    }
                    c if c == quote_char && in_quotes => {
                        // End a quoted path
                        in_quotes = false;
                        file_paths.push(current_path.clone());
                        current_path.clear();
                    }
                    ' ' if !in_quotes => {
                        // Space outside quotes indicates the end of a path
                        if !current_path.is_empty() {
                            file_paths.push(current_path.clone());
                            current_path.clear();
                        }
                    }
                    _ => {
                        // Append characters to the current path
                        current_path.push(c);
                    }
                }
            }
        
            // Add any remaining path
            if !current_path.is_empty() {
                file_paths.push(current_path);
            }
        
            // Process each file path in order and concatenate results
            let mut output = String::new();
            for file_path in file_paths {
                match fs::read_to_string(&file_path) {
                    Ok(content) => output.push_str(&content),
                    Err(err) => eprintln!("cat: {}: {}", file_path, err),
                }
            }
        
            // Print the concatenated output
            print!("{}", output);
        
            // Explicitly print the prompt for the next command
            continue;
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
            continue;
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
        if c == '\\' {
            // Handle escaped characters
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next(); // Consume 'n'
                    }
                    't' => {
                        result.push('\t');
                        chars.next(); // Consume 't'
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next(); // Consume '\'
                    }
                    '"' => {
                        result.push('"');
                        chars.next(); // Consume '"'
                    }
                    '$' => {
                        result.push('$');
                        chars.next(); // Consume '$'
                    }
                    _ => {
                        // Preserve the backslash and the character
                        result.push('\\');
                        result.push(next);
                        chars.next(); // Consume the character
                    }
                }
            } else {
                // Preserve trailing backslash
                result.push('\\');
            }
        } else if c == '$' {
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
            } else {
                result.push('$');
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}
