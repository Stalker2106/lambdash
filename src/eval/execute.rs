use std::{io::Write, process::{self, Child, Stdio}};

use crate::core::{cmdoutput::CmdOutput, core::ShellError};
use crate::eval::eval::ExecutionError;


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

  return Ok(child)
}

pub fn execute_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<CmdOutput, ShellError> {
  match spawn_program(program, args, input) {
      Ok(child) => match child.wait_with_output() {
          Ok(output) => Ok(CmdOutput::from_output(&output)),
          Err(e) => Err(ShellError::Execution(ExecutionError::new(
              1,
              format!("Failed to execute command: {}", e),
          ))),
      },
      Err(error) => Err(error)
  }
}
