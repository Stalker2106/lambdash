use std::{env, fs};

use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;

use crate::core::core::{ShellState};
use crate::core::error::ShellError;
use crate::rendering::autocomplete::render_options;

pub struct AutocompleteState {
    pub index: Option<usize>,
    pub items: Vec<String>
}

pub struct Autocomplete {
    state: Option<AutocompleteState>
}

impl Autocomplete {
    pub fn new() -> Autocomplete {
        return Autocomplete{
            state: None
        };
    }

    pub fn complete(&mut self, state: &mut ShellState, expr: &str) -> Result<Option<String>, ShellError> {
        // Don't complete if input is empty or only whitespace
        if expr.trim().is_empty() {
            return Ok(None);
        }
        
        let mut res: Option<String> = None;
        if let Some(astate) = self.state.as_mut() {
            if let Some(index) = astate.index.as_mut() {
                if *index < astate.items.len() - 1 {
                    *index += 1;
                } else {
                    *index = 0;
                }
                // Reconstruct full command with the selected completion
                if let Some(completion) = astate.items.get(*index) {
                    res = Some(build_full_completion(expr, completion));
                }
            } else {
                astate.index = Some(0);
                if let Some(completion) = astate.items.get(0) {
                    res = Some(build_full_completion(expr, completion));
                }
            }
        } else {
            let completions: Vec<String> = if should_complete_path(&expr) {
                path_completion(&expr)
            } else {
                command_completion(&expr)
            };
            match completions.len() {
                0 => {
                    return Ok(None);
                },
                1 => {
                    return Ok(Some(build_full_completion(expr, &completions[0])));
                },
                _ => {
                    let astate = AutocompleteState {
                        index: None,
                        items: completions
                    };
                    self.state = Some(astate);
                }
            }
        }
        if let Some(astate) = &self.state {
            print_options(state, &astate).unwrap();
        }
        return Ok(res);
    }

    pub fn reset(&mut self, state: &mut ShellState) {
        self.state = None;
        state.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
    }
}

fn build_full_completion(expr: &str, completion: &str) -> String {
    // Get the prefix (everything before the last word being completed)
    let words: Vec<&str> = expr.split_whitespace().collect();
    let has_trailing_space = expr.ends_with(' ') || expr.ends_with('\t');
    
    if has_trailing_space {
        // Completing a new argument after space
        format!("{}{}", expr, completion)
    } else if words.len() > 1 {
        // Replace the last word with the completion
        let last_word = words.last().unwrap();
        let prefix_end = expr.rfind(last_word).unwrap();
        format!("{}{}", &expr[..prefix_end], completion)
    } else {
        // First word - just return the completion
        completion.to_string()
    }
}

fn should_complete_path(expr: &str) -> bool {
    // Split the expression into words to get the last argument
    let words: Vec<&str> = expr.split_whitespace().collect();
    let last_word = words.last().unwrap_or(&"");
    
    // Check if there's a trailing space (means we're starting a new argument)
    let has_trailing_space = expr.ends_with(' ') || expr.ends_with('\t');
    
    // Path completion indicators:
    // 1. Contains path separators (/ or ~)
    // 2. Starts with . (current/parent directory)
    // 3. Multiple words (not the first command word)
    // 4. Trailing space after command (starting new argument)
    
    if last_word.contains('/') || last_word.starts_with('~') {
        return true;
    }
    
    if last_word.starts_with('.') {
        return true;
    }
    
    // If we have multiple words OR trailing space, complete paths for arguments
    if words.len() > 1 || (words.len() == 1 && has_trailing_space) {
        return true;
    }
    
    false
}

fn command_completion(expr: &str) -> Vec<String> {
    let mut available: Vec<String> = Vec::new();
    let mut searchpaths: Vec<String> = Vec::new();
    if let Ok(path) = env::var("PATH") {
        searchpaths = path.split(":").map(String::from).collect();
    }
    for searchpath in searchpaths {
        if let Ok(entries) = fs::read_dir(searchpath) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(path_str) = path.file_name().and_then(|f| f.to_str()) {
                            // Filter commands based on input
                            if path_str.starts_with(expr) {
                                available.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    available.sort();
    return available;
}

fn print_options(state: &mut ShellState, astate: &AutocompleteState) -> Result<(), ShellError>  {
    match render_options(state, astate, state.termsize.1 / 2) {
        Ok(out) => {
            if let Ok(output) = String::from_utf8(out.stdout) {
                state.stdout.queue(Print(output)).unwrap();
                return Ok(());
            } else {
                return Err(ShellError::ExitRequest);
            }
        },
        Err(error) => {
            return Err(error);
        }
    }
}

fn path_completion(expr: &str) -> Vec<String> {
    use std::path::Path;
    let mut available = Vec::new();

    // Extract the last word (the part we're completing)
    let words: Vec<&str> = expr.split_whitespace().collect();
    let has_trailing_space = expr.ends_with(' ') || expr.ends_with('\t');
    
    // If trailing space, we're completing a new empty argument
    let last_word: &str = if has_trailing_space {
        ""
    } else {
        words.last().unwrap_or(&"")
    };

    // Handle tilde expansion
    let expanded_path = if last_word.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            last_word.replacen("~", &home, 1)
        } else {
            last_word.to_string()
        }
    } else {
        last_word.to_string()
    };

    // Split input into directory and prefix
    let path = Path::new(&expanded_path);
    
    let dir = if path.is_dir() && last_word.ends_with('/') {
        path
    } else if let Some(parent) = path.parent() {
        if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        }
    } else {
        Path::new(".")
    };
    let file_prefix = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Get the directory prefix to prepend to completions
    let dir_prefix = if last_word.contains('/') {
        let last_slash = last_word.rfind('/').unwrap();
        &last_word[..=last_slash]
    } else {
        ""
    };

    // Read entries in the directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();

                // Match entries that start with the prefix
                if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    // Skip hidden files unless explicitly requested (prefix starts with .)
                    if !file_prefix.starts_with('.') && name.starts_with('.') {
                        continue;
                    }
                    
                    if name.starts_with(file_prefix) {
                        // Build the completion (just the path part, not the full command)
                        let completion = if entry_path.is_dir() {
                            format!("{}{}/", dir_prefix, name)
                        } else {
                            format!("{}{}", dir_prefix, name)
                        };
                        available.push(completion);
                    }
                }
            }
        }
    }

    // Sort the results
    available.sort();
    return available;
}