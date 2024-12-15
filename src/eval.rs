
use std::env;
use std::process;
use std::process::{ExitStatus, Stdio};
use std::os::unix::process::ExitStatusExt;

use crate::cmdoutput::CmdOutput;
use crate::core::{ShellError, ShellState};
use crate::command::{self, parse_tokens};
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

// Expanding

pub fn expand_variable(state: &mut ShellState, var_name: &str) -> String {
    match var_name {
        "?" => match state.status.code() {
            Some(code) => format!("{}", code),
            None => "".to_string()
        }
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
            _ => {}
        }
    }
}

// Execution

pub fn run_command(state: &mut ShellState, command: &Vec<command::Command>, output: &mut CmdOutput) -> Result<CmdOutput, ShellError> {
    for step in command {
        let cmd = &step.words[0];
        let args = step.words[1..].to_vec();
        match match_builtin(state, cmd, &args, output) {
            Ok(out) => return Ok(out),
            Err(error) => {
                match error {
                    ShellError::NoBuiltin => {
                        match execute(cmd, &args, output) {
                            Ok(out) => {
                                output.combine(&out)
                            },
                            Err(err) => return Err(err)
                        }
                    },
                    error => return Err(error)
                }
            }
        }
    }
    Ok(output.clone())
}

pub fn execute(command: &str, args: &Vec<String>, output: &CmdOutput) -> Result<CmdOutput, ShellError> {
    let mut process = process::Command::new(command);
    process.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // If no input, execute the command normally
    match process.output() {
        Ok(output) => Ok(CmdOutput::from_output(&output)),
        Err(_error) => Err(ShellError::Execution(ExecutionError::new(
            127,
            format!("{}: command not found", command),
        ))),
    }
}

// Eval

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<Option<CmdOutput>, ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                match parse_tokens(&tokens) {
                    Ok(commands) => {
                        let mut output = CmdOutput::from_status(ExitStatus::from_raw(0));
                        for cmd in commands {
                            match run_command(state, &cmd, &mut output) {
                                Ok(out) => {
                                    output.combine(&out);
                                    return Ok(Some(output))
                                },
                                Err(error) => return Err(error)
                            }
                        }
                    },
                    Err(error) => return Err(ShellError::Execution(ExecutionError::new(1, format!("invalid syntax"))))
                }
            }
            return Ok(None)
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}