use std::env;

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

pub fn expand_tokens(state: &mut ShellState, tokens: &mut Vec<Token>) {
  // Iterate over each token in the vector
  for token in tokens.iter_mut() {
      match token {
          Token::Variable(var_name) => {
              *token = Token::Word(expand_variable(state, var_name));
          }
          Token::Word(word) => {
              if word.contains('~') {
                  if let Ok(home) = env::var("HOME") {
                      *token = Token::Word(word.replace("~", &home));
                  }
              }
          }
          _ => {}
      }
  }
}
