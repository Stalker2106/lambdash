use std::env;
use glob::glob;

use crate::core::core::ShellState;
use crate::parser::tokenizer::Token;

pub fn expand_variable(state: &mut ShellState, var_name: &str) -> String {
  match var_name {
      "?" => format!("{}", state.status),
      _ => {
          match env::var(var_name) {
              Ok(var_value) => var_value,
              Err(_) => format!("${}", var_name)
          }
      }
  }
}

pub fn expand_glob(glob_expr: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    // Use glob crate to match the glob expression
    for entry in glob(glob_expr).unwrap() {
        match entry {
            Ok(path) => {
                // Convert Path to String and push a Token::Word to the result vector
                if let Some(path_str) = path.to_str() {
                    tokens.push(Token::Word(path_str.to_string()));
                }
            }
            Err(_) => (),
        }
    }

    tokens
}

pub fn expand_tokens(state: &mut ShellState, tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        let token = &mut tokens[i];

        match token {
            Token::Variable(var_name) => {
                *token = Token::Word(expand_variable(state, var_name));
                i += 1;
            }
            Token::Word(word) => {
                if word.contains('~') {
                    if let Ok(home) = env::var("HOME") {
                        *token = Token::Word(word.replace("~", &home));
                    }
                    i += 1;
                } else if word.contains('*') || word.contains('?') {
                    let glob_results = expand_glob(word);
                    if !glob_results.is_empty() {
                        tokens.splice(i..=i, glob_results.clone());
                        i += glob_results.len();
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1; // Move to the next token for other types
            }
        }
    }
}