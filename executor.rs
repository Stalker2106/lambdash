
use std::env;
use std::process::Stdio;
use std::io::Write;
use std::process::Command;
use std::collections::VecDeque;

use crate::shell::Shell;
use crate::tokenizer::Token;
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

pub fn preprocess_tokens(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Variable(var) => {
                match env::var(var) {
                    Ok(value) => {
                        tokens[i] = Token::Word(value.to_string());
                    },
                    Err(_error) => {
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

pub fn execute_tokens(shell: &mut Shell, tokens: &Vec<Token>) -> Result<u8, ExecutionError> {
    let mut args: VecDeque<&str> = VecDeque::new();
    let mut action: Option<&Token> = None;
    let mut status: u8 = shell.status; 
    for token in tokens {
        match token {
            Token::EndOfInput => {
                if let Some(command) = args.pop_front() {
                    if let Some(code) = match_builtin(&command, &args) {
                        status = code;
                    } else {
                        match execute(command, &args) {
                            Ok(code) => status = code,
                            Err(error) => return Err(error)
                        }
                    }
                }
                args.clear();
            },
            Token::Word(w) => args.push_back(w.as_str()),
            Token::Redirection(r) => {
                if action.is_some() {
                    return Err(ExecutionError::new(status, format!("invalid redirection {}", r)));
                }
                action = Some(token);
            }
            Token::Pipe => {
                if action.is_some() {
                    return Err(ExecutionError::new(status, format!("invalid pipe")));
                }
                action = Some(token);
            }
            _ => continue
        }
    }
    Ok(status)
}

pub fn execute(command: &str, args: &VecDeque<&str>) -> Result<u8, ExecutionError> {
    match Command::new(command).args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
        Ok(output) => {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            match output.status.code() {
                Some(code) => return Ok(code as u8),
                None => return Ok(0)
            };
        },
        Err(_error) => {
            return Err(ExecutionError::new(127, format!("{}: command not found", command)));
        }
    }
}
