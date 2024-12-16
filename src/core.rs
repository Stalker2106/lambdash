use std::io::Write;
use std::collections::HashMap;

use crate::config::{ShellConfig, load};
use crate::eval::ExecutionError;
use crate::tokenizer::TokenizationError;

#[derive(Debug)]
pub enum ShellError {
    Tokenization(TokenizationError),
    Execution(ExecutionError),
    NoBuiltin,
    ExitRequest
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
    pub status: i32,
    pub ps1pos: (u16, u16),
    pub aliases: HashMap<String, String>,
    pub config: ShellConfig,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write
}

impl<'a> ShellState<'a> {
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> ShellState<'a> {
        ShellState {
            status: 0,
            ps1pos: (0,0),
            aliases: HashMap::new(),
            config: load(),
            stdout: out,
            stderr: err,
        }
    }
}
