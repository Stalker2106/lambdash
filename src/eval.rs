
use std::env;
use std::process::Stdio;
use std::process::Command;
use std::collections::VecDeque;

use crate::core::{ShellError, ShellState};
use crate::tokenizer::{Token, tokenize};
use crate::builtins::match_builtin;

#[derive(Debug)]
pub struct ExecutionError {
    pub status: u8,
    pub details: String
}

impl ExecutionError {
    fn new(code: u8, msg: String) -> ExecutionError {
        ExecutionError{status: code, details: msg.to_string()}
    }
}

pub fn expand_variable(state: &mut ShellState, var: &str) -> Option<String> {
    match var {
        "?" => Some(state.status.to_string()),
        _ => env::var(var).ok(),
    }
}

pub fn expand_tokens(state: &mut ShellState, tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Variable(token_value) => {
                match expand_variable(state, &token_value) {
                    Some(value) => {
                        tokens[i] = Token::Word(value.to_string());
                    },
                    None => {
                        tokens.remove(i);
                        continue; // Skip incrementing since we removed
                    }
                }
            },
            _ => ()
        }
        i += 1;
    }
}

pub fn eval_tokens(state: &mut ShellState, tokens: &Vec<Token>) -> Result<Option<u8>, ShellError> {
    let mut args: VecDeque<&str> = VecDeque::new();
    let mut action: Option<&Token> = None;
    let mut status: Option<u8> = None;
    for token in tokens {
        match token {
            Token::EndOfInput => {
                if let Some(command) = args.pop_front() {
                    match match_builtin(state, &command, &args) {
                        Ok(res) => {
                            if res.is_some() {
                                status = res;
                            } else {
                                match execute(state, command, &args) {
                                    Ok(code) => status = Some(code),
                                    Err(error) => return Err(error)
                                }
                            }
                        },
                        Err(error) => return Err(error)
                    }
                }
                args.clear();
            },
            Token::Word(w) => args.push_back(w.as_str()),
            Token::Redirection(r) => {
                if action.is_some() {
                    return Err(ShellError::Execution(ExecutionError::new(123, format!("invalid redirection {}", r))));
                }
                action = Some(token);
            }
            Token::Pipe => {
                if action.is_some() {
                    return Err(ShellError::Execution(ExecutionError::new(127, format!("invalid pipe"))));
                }
                action = Some(token);
            }
            _ => continue
        }
    }
    Ok(status)
}

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<Option<u8>, ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                return eval_tokens(state, &tokens)
            }
            return Ok(None)
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}

pub fn execute(state: &mut ShellState, command: &str, args: &VecDeque<&str>) -> Result<u8, ShellError> {
    match Command::new(command)
                  .args(args)
                  .stdout(Stdio::piped())
                  .stderr(Stdio::piped())
                  .output() {
        Ok(output) => {
            state.stdout.write_all(&output.stdout).unwrap();
            state.stdout.write_all(&output.stderr).unwrap();
            match output.status.code() {
                Some(code) => return Ok(code as u8),
                None => return Ok(0)
            };
        },
        Err(_error) => {
            return Err(ShellError::Execution(ExecutionError::new(127, format!("{}: command not found", command))));
        }
    }
}
