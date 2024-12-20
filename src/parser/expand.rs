use std::env;
use glob::glob;
use crate::core::core::ShellState;
use crate::parser::tokenizer::Token;

/// Expands a variable based on the shell state and environment variables.
///
/// # Arguments
/// - `state`: The current shell state.
/// - `var_name`: The name of the variable to expand.
///
/// # Returns
/// - The expanded variable value as a string. If the variable does not exist, returns `$var_name`.
pub fn expand_variable(state: &mut ShellState, var_name: &str) -> String {
    match var_name {
        "?" => format!("{}", state.status),
        _ => match env::var(var_name) {
            Ok(var_value) => var_value,
            Err(_) => format!("${}", var_name),
        },
    }
}

/// Expands a glob expression into a list of `Token::Word` representing matching file paths.
///
/// # Arguments
/// - `glob_expr`: The glob expression to expand (e.g., `*.rs`, `dir/*`).
///
/// # Returns
/// - A vector of `Token::Word` containing the file paths matching the glob expression.
/// - If no matches are found, returns an empty vector.
pub fn expand_glob(glob_expr: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    match glob(glob_expr) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(path) = entry {
                    if let Some(path_str) = path.to_str() {
                        tokens.push(Token::Word(path_str.to_string()));
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error processing glob pattern '{}': {}", glob_expr, err);
        }
    }

    tokens
}

/// Expands tokens in-place, resolving variables, tilde (`~`) for home directories, and glob expressions.
///
/// # Arguments
/// - `state`: The current shell state.
/// - `tokens`: The vector of tokens to expand.
pub fn expand_tokens(state: &mut ShellState, tokens: &mut Vec<Token>) {
    let mut expanded_tokens = Vec::new();

    for token in tokens.drain(..) {
        match token {
            // Expand variables (e.g., `$VAR`).
            Token::Variable(var_name) => {
                expanded_tokens.push(Token::Word(expand_variable(state, &var_name)));
            }

            // Handle words for tilde (`~`) and glob patterns (`*`, `?`).
            Token::Word(word) => {
                if word.starts_with('~') {
                    // Replace `~` with the home directory.
                    let expanded = match env::var("HOME") {
                        Ok(home) => word.replace("~", &home),
                        Err(_) => word.clone(), // Leave unchanged if `HOME` is not set.
                    };
                    expanded_tokens.push(Token::Word(expanded));
                } else if word.contains('*') || word.contains('?') {
                    // Expand glob patterns.
                    let glob_results = expand_glob(&word);
                    if !glob_results.is_empty() {
                        expanded_tokens.extend(glob_results);
                    } else {
                        expanded_tokens.push(Token::Word(word)); // Leave unchanged if no matches.
                    }
                } else {
                    expanded_tokens.push(Token::Word(word));
                }
            }

            // Keep other tokens as-is.
            _ => expanded_tokens.push(token),
        }
    }

    *tokens = expanded_tokens;
}
