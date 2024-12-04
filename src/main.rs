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
        
            // Initialize variables
            let mut result = String::new();
            let mut current_segment = String::new();
            let mut in_quotes = false;
            let mut quote_char = '\0';
            let mut needs_space = false;
        
            let mut chars = echo_message.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '\'' | '"' => {
                        if in_quotes && c == quote_char {
                            // End of quoted segment
                            in_quotes = false;
                            if !current_segment.is_empty() {
                                if !result.is_empty() && needs_space {
                                    result.push(' ');
                                    needs_space = false;
                                }
                                if quote_char == '"' {
                                    let interpreted_segment = interpret_special_characters(&current_segment);
                                    result.push_str(&interpreted_segment);
                                } else {
                                    result.push_str(&current_segment);
                                }
                                current_segment.clear();
                                // Do not set needs_space here
                            }
                        } else if !in_quotes {
                            // Start of quoted segment
                            in_quotes = true;
                            quote_char = c;
                            // Check if we need to add a space before starting a new segment
                            if needs_space && !result.is_empty() {
                                result.push(' ');
                                needs_space = false;
                            }
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
                                    // Trailing backslash inside double quotes
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
                            if !result.is_empty() {
                                result.push(' ');
                            }
                            result.push_str(&current_segment);
                            current_segment.clear();
                            
                        }
                        needs_space = true;
                        
                    }
                    _ => {
                        // Before adding to current_segment, check needs_space
                        if needs_space && !result.is_empty() && current_segment.is_empty() {
                            result.push(' ');
                            needs_space = false;
                        }
                        current_segment.push(c);
                    }
                }
            }
        
            // Add any remaining segment
            if !current_segment.is_empty() {
                if !result.is_empty() && needs_space {
                    result.push(' ');
                }
                result.push_str(&current_segment);
            }
        
            // Print the result
            println!("{}", result);
        
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
            // Parse the command line into tokens, respecting quotes and escapes
            let tokens = parse_command_line(command);

            if let Some(command_name) = tokens.get(0) {
                let args = &tokens[1..];

                // Attempt to find the executable
                let executable_path = if command_name.contains('/') {
                    // If the command contains a slash, treat it as a path
                    command_name.clone()
                } else if let Some(path) = find_in_path(command_name) {
                    path
                } else {
                    println!("{}: command not found", command_name);
                    continue;
                };

                // Execute the command with arguments
                match run_command(&executable_path, &args) {
                    Ok(output) => print!("{}", output),
                    Err(err) => eprintln!("{}", err),
                }
            }
            continue;
        }

    }
}

// Function to search for an executable in the system's PATH
fn find_in_path(executable: &str) -> Option<String> {
    if executable.contains('/') {
        if fs::metadata(executable).is_ok() {
            return Some(executable.to_string());
        }
    } else if let Ok(path_var) = env::var("PATH") {
        for path in path_var.split(':') {
            let full_path = format!("{}/{}", path, executable);
            if fs::metadata(&full_path).is_ok() {
                return Some(full_path);
            }
        }
    }
    None
}


// Function to run a command with arguments
fn run_command(executable: &str, args: &[String]) -> Result<String, String> {
    let output = Command::new(executable)
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", executable, e))?;

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

fn parse_command_line(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if in_quotes {
                    if quote_char == '"' {
                        // Inside double quotes
                        if let Some(&next_char) = chars.peek() {
                            match next_char {
                                '\\' | '"' | '$' | '`' | '\n' => {
                                    chars.next(); // Consume the next character
                                    current_token.push(next_char);
                                }
                                _ => {
                                    // Backslash is preserved
                                    current_token.push('\\');
                                }
                            }
                        } else {
                            // Trailing backslash, preserve it
                            current_token.push('\\');
                        }
                    } else {
                        // Inside single quotes, backslash is literal
                        current_token.push('\\');
                    }
                } else {
                    // Outside quotes, backslash escapes the next character
                    if let Some(next_char) = chars.next() {
                        current_token.push(next_char);
                    } else {
                        // Trailing backslash, preserve it
                        current_token.push('\\');
                    }
                }
            }
            '\'' | '"' => {
                if in_quotes {
                    if c == quote_char {
                        // End of quoted segment
                        in_quotes = false;
                    } else {
                        // Different quote inside quotes is literal
                        current_token.push(c);
                    }
                } else {
                    // Start of quoted segment
                    in_quotes = true;
                    quote_char = c;
                }
            }
            ' ' | '\t' if !in_quotes => {
                // Token separator
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(c);
            }
        }
    }

    // Add any remaining token
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}
