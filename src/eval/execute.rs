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

pub fn spawn_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>, capture_output: bool) -> Result<Child, ShellError> {
  let mut process = process::Command::new(program);

  let stdin = if input.is_some() {
      Stdio::piped()
  } else {
      Stdio::inherit()
  };

  process.args(args)
      .stdin(stdin)
      .stdout(if capture_output { Stdio::piped() } else { Stdio::inherit() })
      .stderr(if capture_output { Stdio::piped() } else { Stdio::inherit() });

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

pub fn execute_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>, capture_output: bool) -> Result<CmdOutput, ShellError> {
  match spawn_program(program, args, input, capture_output) {
      Ok(mut child) => {
          if capture_output {
              match child.wait_with_output() {
                  Ok(output) => Ok(CmdOutput::from_output(&output, capture_output)),
                  Err(_) => Err(ShellError::Execution(ExecutionError::ExecutionFailed)),
              }
          } else {
              match child.wait() {
                  Ok(status) => {
                      let code = match status.code() {
                          Some(code) => code,
                          None => 1,
                      };
                      Ok(CmdOutput::from_status(code))
                  }
                  Err(_) => Err(ShellError::Execution(ExecutionError::ExecutionFailed)),
              }
          }
      }
      Err(error) => Err(error)
  }
}
