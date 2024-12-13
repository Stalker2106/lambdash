
use std::env;
use std::process::{Command, ExitStatus, Stdio};
use std::os::unix::process::ExitStatusExt;

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

pub fn run_command(command: &Vec<&str>, input: &Option<CmdOutput>) -> Result<Option<CmdOutput>, ShellError> {
    let args = command.iter().skip(1).copied().collect::<Vec<&str>>();
    match match_builtin(&command[0], &args, &input) {
        Ok(builtin_out) => {
            if builtin_out.is_some() {
                return Ok(builtin_out);
            } else {
                match execute(command[0], &args, &input) {
                    Ok(output) => return Ok(output),
                    Err(err) => return Err(err)
                }
            }
        },
        Err(error) => return Err(error)
    }
}

pub fn eval_tokens(tokens: &Vec<Token>) -> Result<Option<CmdOutput>, ShellError> {
    let mut command: Vec<&str> = Vec::new();
    let mut action: Option<&Token> = None;
    let mut output: Option<CmdOutput> = None;
    let mut token_it = tokens.iter().peekable();
    while let Some(token) = token_it.next() {
        match token {
            Token::Word(w) => {
                command.push(w.as_str());
                if !matches!(token_it.peek(), Some(Token::Word(_))) {
                    match run_command(&command, &output) {
                        Ok(res) => output = res,
                        Err(err) => return Err(err)
                    }
                    command.clear();
                    action = None;
                }
            },
            Token::Redirection(r) => {
                if action.is_some() {
                    return Err(ShellError::Execution(ExecutionError::new(123, format!("invalid redirection {}", r))));
                }
                action = Some(&token);
            }
            Token::Pipe => {
                if action.is_some() {
                    return Err(ShellError::Execution(ExecutionError::new(127, format!("invalid pipe"))));
                }
                action = Some(&token);
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

pub fn execute(command: &str, args: &Vec<&str>, input: &Option<CmdOutput>) -> Result<Option<CmdOutput>, ShellError> {
    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Handle piping input into stdin if `input` is Some
    if let Some(input_data) = input {
        if let Some(stdin_data) = &input_data.stdout {
            cmd.stdin(Stdio::piped());
            let mut child = cmd.spawn().map_err(|_error| {
                ShellError::Execution(ExecutionError::new(127, format!("{}: command not found", command)))
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(stdin_data).map_err(|_error| {
                    ShellError::Execution(ExecutionError::new(1, "Failed to write to stdin".to_string()))
                })?;
            }

            let output = child.wait_with_output().map_err(|_error| {
                ShellError::Execution(ExecutionError::new(1, "Failed to read command output".to_string()))
            })?;

            return Ok(Some(CmdOutput::from_output(&output)));
        }
    }
    // If no input, execute the command normally
    match cmd.output() {
        Ok(output) => Ok(Some(CmdOutput::from_output(&output))),
        Err(_error) => Err(ShellError::Execution(ExecutionError::new(
            127,
            format!("{}: command not found", command),
        ))),
    }
}
