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