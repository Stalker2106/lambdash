use std::io::Cursor;

use crossterm::{style::Print, QueueableCommand};

use crate::{eval::{builtins::BuiltinError, execute::ExecutionError, expression::ParserError}, parser::tokenizer::TokenizationError};

pub trait StatusEnum {
  fn status(&self) -> u16;
}

#[derive(Debug)]
pub enum ShellError {
    Tokenization(TokenizationError),
    Execution(ExecutionError),
    Builtin(BuiltinError),
    Parser(ParserError),
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

impl From<BuiltinError> for ShellError {
    fn from(err: BuiltinError) -> Self {
        ShellError::Builtin(err)
    }
}

impl From<ParserError> for ShellError {
    fn from(err: ParserError) -> Self {
        ShellError::Parser(err)
    }
}

impl ShellError {
  pub fn to_output(&self, input: &str) -> Vec<u8> {
      match self {
          ShellError::Tokenization(error) => print_tokenization_error(error),
          ShellError::Execution(error) => print_execution_error(error, input),
          ShellError::Builtin(error) => print_builtin_error(error),
          ShellError::Parser(error) => print_parser_error(error),
          ShellError::NoBuiltin => "The requested builtin command was not found.".as_bytes().to_vec(),
          ShellError::ExitRequest => "The shell received an exit request.".as_bytes().to_vec(),
      }
  }

  pub fn status(&self) -> u16 {
      match self {
          ShellError::Tokenization(error) => error.status(),
          ShellError::Execution(error) => error.status(),
          ShellError::Builtin(error) => error.status,
          ShellError::Parser(error) => error.status(),
          ShellError::NoBuiltin => 127,
          ShellError::ExitRequest => 0,
      }
  }
}

pub fn print_tokenization_error(error: &TokenizationError) -> Vec<u8> {
  let mut output: Vec<u8> = Vec::new();
  let mut cursor = Cursor::new(&mut output);
  cursor.queue(Print(format!("{:?}", error))).unwrap();
  return output;
}

pub fn print_execution_error(error: &ExecutionError, input: &str) -> Vec<u8>  {
  let mut output: Vec<u8> = Vec::new();
  let mut cursor = Cursor::new(&mut output);
  match error {
    ExecutionError::CommandNotFound => {
      cursor.queue(Print(&format!("{}: command not found", input))).unwrap();
    },
    _ => {
      cursor.queue(Print(format!("{:?}", error))).unwrap();
    }
  }
  return output;
}

pub fn print_builtin_error(error: &BuiltinError) -> Vec<u8>  {
  let mut output: Vec<u8> = Vec::new();
  let mut cursor = Cursor::new(&mut output);
  cursor.queue(Print(format!("{}", error.message))).unwrap();
  return output;
}

pub fn print_parser_error(error: &ParserError) -> Vec<u8>  {
  let mut output: Vec<u8> = Vec::new();
  let mut cursor = Cursor::new(&mut output);
  cursor.queue(Print(format!("{:?}", error))).unwrap();
  return output;
}