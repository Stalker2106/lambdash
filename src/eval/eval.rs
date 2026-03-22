use crate::core::error::ShellError;
use crate::crossterm::QueueableCommand;
use crate::crossterm::style::Print;

use crate::eval::execute::{execute_program, spawn_program};
use crate::parser::expand::expand_tokens;
use crate::eval::expression::parse_tokens;
use crate::core::cmdoutput::CmdOutput;
use crate::core::core::ShellState;
use crate::eval::expression::ExpressionGroup;
use crate::eval::redirections::handle_input_redirections;
use crate::eval::redirections::handle_output_redirections;
use crate::parser::tokenizer::tokenize;
use crate::eval::builtins::match_builtin;

pub fn eval_group(state: &mut ShellState, group: &ExpressionGroup) -> Result<Option<CmdOutput>, ShellError> {
    let mut output: Option<CmdOutput> = None;
    let mut local_env: Vec<(String, String)> = Vec::new();
    for (i, expr) in group.expressions.iter().enumerate() {
        let mut first_word = 0;
        // Handle assignment words
        while first_word < expr.words.len() {
            if let Some((key, value)) = word_is_assignment(&expr.words[first_word]) {
                local_env.push((key.to_string(), value.to_string()));
                first_word += 1;
            } else {
                break;
            }
        }
        // Check if there's no command left in group, and transfer to state if so
        if first_word == expr.words.len() {
            // All words were assignments, transfer to state
            for (key, value) in local_env.drain(..) {
                state.variables.insert(key, value);
            }
            continue; // no command to run
        }
        // Handle group normally
        let is_last = i == group.expressions.len() - 1;
        let program = &expr.words[first_word];
        let args = expr.words[first_word + 1..].to_vec();
        let mut input:  Option<Vec<u8>> = None;
        if let Ok(res) = handle_input_redirections(&expr.inputs) {
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
                        if expr.background {
                            let bg_input = input.or(Some(vec![]));
                            // For background jobs, do not capture stdout/stderr, so output is written directly
                            // to the current terminal and the process can run without pipe buffering.
                            match spawn_program(program, &args, &bg_input, Some(&local_env), false) {
                                Ok(child) => {
                                    let pid = child.id();
                                    let job_id = state.jobs.len() + 1;
                                    let job_cmd = format!("{} {}", program, args.join(" "));
                                    state.jobs.push((job_id, job_cmd, child));
                                    state.stdout.queue(Print(format!("[{}] {} {}\n", job_id, pid, program))).unwrap();
                                    output = None; // Background jobs don't produce capture output
                                },
                                Err(err) => return Err(err)
                            }
                        } else {
                            match execute_program(program, &args, &input, Some(&local_env), !is_last) {
                                Ok(out) => {
                                    if let Ok(res) = handle_output_redirections(&expr.outputs, &out.stdout) {
                                        if !res {
                                            output = Some(out);
                                        }
                                    }
                                },
                                Err(err) => return Err(err)
                            }
                        }
                    },
                    error => return Err(error)
                }
            }
        }
    }
    return Ok(output);
}

// Eval

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<(), ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                match parse_tokens(&tokens) {
                    Ok(groups) => {
                        for group in groups {
                            match &group.gtype {
                                _ => {
                                    match eval_group(state, &group) {
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
                            }
                        }
                        return Ok(());
                    },
                    Err(error) => return Err(ShellError::Parser(error))
                }
            }
            return Ok(())
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}

fn word_is_assignment(word: &str) -> Option<(&str, &str)> {
    let (lhs, rhs) = word.split_once('=')?;
    if !lhs.is_empty()
        && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
        && lhs.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
    {
        return Some((lhs, rhs));
    }
    None
}