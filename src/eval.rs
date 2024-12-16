use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::env;
use std::process;
use std::process::Stdio;
use std::fs::OpenOptions;

use crate::crossterm::QueueableCommand;
use crate::crossterm::style::Print;

use crate::expression::Expression;
use crate::expression::Redirection;
use crate::expression::parse_tokens;
use crate::cmdoutput::CmdOutput;
use crate::core::{ShellError, ShellState};
use crate::fsio::FSError;
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

pub fn run_command(state: &mut ShellState, command: &Vec<Expression>) -> Result<Option<CmdOutput>, ShellError> {
    let mut output: Option<CmdOutput> = None;
    for step in command {
        let program = &step.words[0];
        let args = step.words[1..].to_vec();
        let mut input:  Option<Vec<u8>> = None;
        if let Ok(res) = handle_input_redirections(&step.inputs) {
            if res.is_some() {
                input = res;
            } else if let Some(out) = &output {
                input = Some(out.stdout.clone());
            }
        }
        match match_builtin(state, program, &args, &input) {
            Ok(out) => {
                output = Some(out);
            }
            Err(error) => {
                match error {
                    ShellError::NoBuiltin => {
                        match execute_program(program, &args, &input) {
                            Ok(out) => {
                                if !handle_output_redirections(&step.outputs, &out) {
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

fn handle_input_redirections(redirections: &Vec<Redirection>) -> Result<Option<Vec<u8>>, FSError> {
    for (index, redirection) in redirections.iter().enumerate() {
        //Skip all input redirections until last...

        // If this is the last redirection, read and return input
        if index == redirections.len() - 1 {
            if let Ok(mut file) = File::open(&redirection.target) {
                let mut content = String::new();
                if let Ok(_) = file.read_to_string(&mut content) {
                    return Ok(Some(content.as_bytes().to_vec()));
                }
            }
        }
    }
    return Ok(None);
}

fn handle_output_redirections(redirections: &Vec<Redirection>, output: &CmdOutput) -> bool {
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
            file.write_all(&output.stdout).unwrap();
            return true;
        }
    }
    return false;
}

pub fn execute_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<CmdOutput, ShellError> {
    let mut process = process::Command::new(program);
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
                format!("{}: command not found", program),
            )))
        }
    };

    // If there's input, write it to the child's stdin
    if let Some(input_data) = input {
        if let Some(stdin) = child.stdin.as_mut() {
            if let Err(e) = stdin.write_all(&input_data) {
                return Err(ShellError::Execution(ExecutionError::new(
                    1,
                    format!("Failed to write to stdin: {}", e),
                )));
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

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<(), ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                match parse_tokens(&tokens) {
                    Ok(commands) => {
                        for cmd in commands {
                            match run_command(state, &cmd) {
                                Ok(out) => {
                                    if let Some(cmd_output) = out {
                                        if let Some(status) = cmd_output.status {
                                            state.status = status;
                                        }
                                        if let Ok(cmd_out) = String::from_utf8(cmd_output.stdout) {
                                            state.stdout.queue(Print(cmd_out)).unwrap();
                                        }
                                        if let Ok(cmd_err) = String::from_utf8(cmd_output.stderr) {
                                            state.stderr.queue(Print(cmd_err)).unwrap();
                                        }
                                    }
                                }
                                Err(error) => return Err(error)
                            }
                        }
                        return Ok(());
                    },
                    Err(error) => return Err(ShellError::Execution(ExecutionError::new(1, format!("invalid syntax"))))
                }
            }
            return Ok(())
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}