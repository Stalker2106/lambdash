use std::io::{Write};
use crate::{eval::ExecutionError, tokenizer::TokenizationError};

pub enum ShellError {
    Tokenization(TokenizationError),
    Execution(ExecutionError)
}

impl From<ExecutionError> for ShellError {
    fn from(err: ExecutionError) -> Self {
        ShellError::Execution(err)
    }
}

impl From<TokenizationError> for ShellError {
    fn from(err: TokenizationError) -> Self {
        ShellError::Tokenization(err)
    }
}

pub struct ShellState<'a> {
    pub status: u8,
    pub running: bool,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write
}

impl<'a> ShellState<'a> {
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> ShellState<'a> {
        ShellState {
            status: 0,
            running: true,
            stdout: out,
            stderr: err,
        }
    }
}
