
use std::env;
use std::process::{ExitStatus, Stdio, Command};
use std::os::unix::process::ExitStatusExt;
use std::collections::VecDeque;

use crate::cmdoutput::CmdOutput;
use crate::core::{ShellError, ShellState};
use crate::tokenizer::{Token, tokenize};
use crate::builtins::match_builtin;

#[derive(Debug)]
pub struct ExecutionError {
    pub status: ExitStatus,
    pub details: String
}

impl ExecutionError {
    pub fn new(code: i32, msg: String) -> ExecutionError {
        ExecutionError{status: ExitStatus::from_raw(code), details: msg.to_string()}
    }
}

pub fn expand_variable(state: &mut ShellState, var: &str) -> Option<String> {
    match var {
        "?" => match state.status.code() {
            Some(code) => Some(format!("{}", code)),
            None => Some("".to_string())
        }
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

pub fn eval_tokens(tokens: &Vec<Token>) -> Result<Option<CmdOutput>, ShellError> {
    let mut args: VecDeque<&str> = VecDeque::new();
    let mut action: Option<&Token> = None;
    let mut output: Option<CmdOutput> = None;
    for token in tokens {
        match token {
            Token::EndOfInput => {
                if let Some(command) = args.pop_front() {
                    match match_builtin(&command, &args) {
                        Ok(builtin_out) => {
                            if builtin_out.is_some() {
                                output = builtin_out;
                            } else {
                                match execute(command, &args) {
                                    Ok(out) => output = out,
                                    Err(err) => return Err(err)
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
    return Ok(output);
}

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<Option<CmdOutput>, ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                return eval_tokens(&tokens)
            }
            return Ok(None)
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}

pub fn execute(command: &str, args: &VecDeque<&str>) -> Result<Option<CmdOutput>, ShellError> {
    match Command::new(command)
                  .args(args)
                  .stdout(Stdio::piped())
                  .stderr(Stdio::piped())
                  .output() {
        Ok(output) => Ok(Some(CmdOutput::from_output(&output))),
        Err(_error) => Err(ShellError::Execution(ExecutionError::new(127, format!("{}: command not found", command))))
    }
}
