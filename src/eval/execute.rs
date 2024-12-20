use std::{io::Write, process::{self, Child, Stdio}};

use crate::core::{cmdoutput::CmdOutput, error::{ShellError, StatusEnum}};

#[derive(Debug, Copy, Clone)]
pub enum ExecutionError {
    CommandNotFound = 127,
    ExecutionFailed = 128,
    FailedToWriteStdin = 129
}

impl StatusEnum for ExecutionError {
    fn status(&self) -> u16 {
        *self as u16
    }
}

pub fn spawn_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<Child, ShellError> {
  let mut process = process::Command::new(program);
  process.args(args)
      .stdin(Stdio::piped()) // Allow piping input
      .stdout(Stdio::piped()) // Capture stdout
      .stderr(Stdio::piped()); // Capture stderr

  // Spawn the process
  let mut child = match process.spawn() {
      Ok(child) => child,
      Err(_error) => {
          return Err(ShellError::Execution(ExecutionError::CommandNotFound));
      }
  };

  // If there's input, write it to the child's stdin
  if let Some(input_data) = input {
      if let Some(stdin) = child.stdin.as_mut() {
          if let Err(_) = stdin.write_all(&input_data) {
              return Err(ShellError::Execution(ExecutionError::FailedToWriteStdin));
          }
      }
  }

  return Ok(child)
}

pub fn execute_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<CmdOutput, ShellError> {
  match spawn_program(program, args, input) {
      Ok(child) => match child.wait_with_output() {
          Ok(output) => Ok(CmdOutput::from_output(&output)),
          Err(_) => Err(ShellError::Execution(ExecutionError::ExecutionFailed)),
      },
      Err(error) => Err(error)
  }
}
