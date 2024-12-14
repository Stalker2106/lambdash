use std::io::Write;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;
use std::collections::HashMap;

use crate::config::{ShellConfig, load};
use crate::eval::ExecutionError;
use crate::tokenizer::TokenizationError;

#[derive(Debug)]
pub enum ShellError {
    Tokenization(TokenizationError),
    Execution(ExecutionError),
    ExitRequest()
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
    pub status: ExitStatus,
    pub ps1pos: (u16, u16),
    pub aliases: HashMap<String, String>,
    pub config: ShellConfig,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write
}

impl<'a> ShellState<'a> {
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> ShellState<'a> {
        ShellState {
            status: ExitStatus::from_raw(0),
            ps1pos: (0,0),
            aliases: HashMap::new(),
            config: load(),
            stdout: out,
            stderr: err,
        }
    }
}
