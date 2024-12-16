use std::io::Write;
use std::env;
use std::process;
use std::process::Stdio;
use std::fs::OpenOptions;

use crate::cmdoutput::CmdOutput;
use crate::command::Redirection;
use crate::core::{ShellError, ShellState};
use crate::command::{self, parse_tokens};
use crate::io::FSError;
use crate::tokenizer::RedirectionType;
use crate::tokenizer::{Token, tokenize};
use crate::builtins::match_builtin;

#[derive(Debug)]
pub struct ExecutionError {
    pub status: i32,
    pub details: String
}

impl ExecutionError {
    pub fn new(code: i32, msg: String) -> ExecutionError {
        ExecutionError{status: code, details: msg.to_string()}
    }
}

// Expanding

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

// Execution

pub fn run_command(state: &mut ShellState, command: &Vec<command::Command>) -> Result<Option<CmdOutput>, ShellError> {
    let mut output: Option<CmdOutput> = None;
    for step in command {
        let cmd = &step.words[0];
        let args = step.words[1..].to_vec();
        match match_builtin(state, cmd, &args, &output) {
            Ok(out) => {
                output = Some(out);
            }
            Err(error) => {
                match error {
                    ShellError::NoBuiltin => {
                        match execute(cmd, &args, &output) {
                            Ok(out) => {
                                if !handle_redirections(&step.redirections, &out) {
                                    output = Some(out);
                                }
                            },
                            Err(err) => return Err(err)
                        }
                    },
                    error => return Err(error)
                }
            }
        }
    }
    return Ok(output);
}

fn handle_redirections(redirections: &Vec<Redirection>, output: &CmdOutput) -> bool {
    for (index, redirection) in redirections.iter().enumerate() {
        // Create or open the file based on the redirection type
        let mut file = match redirection.rtype {
            RedirectionType::Output => {
                OpenOptions::new().create(true).write(true).truncate(true).open(&redirection.target).unwrap()
            }
            RedirectionType::Append => {
                // Append to the file
                OpenOptions::new().create(true).write(true).append(true).open(&redirection.target).unwrap()
            }
            _ => OpenOptions::new().create(true).write(true).append(true).open(&redirection.target).unwrap()
        };

        // If this is the last redirection, write content to the file
        if index == redirections.len() - 1 {
            if let Some(stdout) = &output.stdout {
                file.write_all(&stdout).unwrap();
            }
            return true;
        }
    }
    return false;
}

pub fn execute(command: &str, args: &Vec<String>, input: &Option<CmdOutput>) -> Result<CmdOutput, ShellError> {
    let mut process = process::Command::new(command);
    process.args(args)
        .stdin(Stdio::piped()) // Allow piping input
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()); // Capture stderr

    // Spawn the process
    let mut child = match process.spawn() {
        Ok(child) => child,
        Err(_error) => {
            return Err(ShellError::Execution(ExecutionError::new(
                127,
                format!("{}: command not found", command),
            )))
        }
    };

    // If there's input, write it to the child's stdin
    if let Some(input_data) = input {
        if let Some(input_stdout) = &input_data.stdout {
            if let Some(stdin) = child.stdin.as_mut() {
                if let Err(e) = stdin.write_all(&input_stdout) {
                    return Err(ShellError::Execution(ExecutionError::new(
                        1,
                        format!("Failed to write to stdin: {}", e),
                    )));
                }
            }
        }
    }

    // Wait for the child process to complete and capture the output
    match child.wait_with_output() {
        Ok(output) => Ok(CmdOutput::from_output(&output)),
        Err(e) => Err(ShellError::Execution(ExecutionError::new(
            1,
            format!("Failed to execute command: {}", e),
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
                        let mut output = CmdOutput::new();
                        for cmd in commands {
                            match run_command(state, &cmd) {
                                Ok(cmdout) => {
                                    if let Some(out) = cmdout {
                                        output.combine(&out);
                                    }
                                }
                                Err(error) => return Err(error)
                            }
                        }
                        return Ok(Some(output));
                    },
                    Err(error) => return Err(ShellError::Execution(ExecutionError::new(1, format!("invalid syntax"))))
                }
            }
            return Ok(None)
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}