use crate::core::error::ShellError;
use crate::crossterm::QueueableCommand;
use crate::crossterm::style::Print;

use crate::eval::execute::execute_program;
use crate::parser::expand::expand_tokens;
use crate::eval::expression::parse_tokens;
use crate::core::cmdoutput::CmdOutput;
use crate::core::core::ShellState;
use crate::eval::expression::ExpressionGroup;
use crate::eval::redirections::handle_input_redirections;
use crate::eval::redirections::handle_output_redirections;
use crate::parser::tokenizer::tokenize;
use crate::eval::builtins::match_builtin;

pub fn run_command(state: &mut ShellState, group: &ExpressionGroup) -> Result<Option<CmdOutput>, ShellError> {
    let mut output: Option<CmdOutput> = None;
    for expr in &group.expressions {
        let program = &expr.words[0];
        let args = expr.words[1..].to_vec();
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
                        match execute_program(program, &args, &input) {
                            Ok(out) => {
                                if let Ok(res) = handle_output_redirections(&expr.outputs, &out.stdout) {
                                    if !res {
                                        output = Some(out);
                                    }
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

// Eval

pub fn eval_expr(state: &mut ShellState, expr: &String) -> Result<(), ShellError> {
    match tokenize(expr) {
        Ok(mut tokens) => {
            if tokens.len() > 0 {
                expand_tokens(state, &mut tokens);
                match parse_tokens(&tokens) {
                    Ok(groups) => {
                        for group in groups {
                            match run_command(state, &group) {
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
                    Err(error) => return Err(ShellError::Parser(error))
                }
            }
            return Ok(())
        },
        Err(error) => return Err(ShellError::Tokenization(error))
    };
}