use std::{io::Write, process::{self, Child, Stdio}};

use crate::core::{cmdoutput::CmdOutput, error::{ShellError, StatusEnum}};

#[derive(Debug, Clone)]
pub enum ExecutionError {
    CommandNotFound(String),
    ExecutionFailed,
    FailedToWriteStdin
}

impl StatusEnum for ExecutionError {
    fn status(&self) -> u16 {
        match self {
            ExecutionError::CommandNotFound(_) => 127,
            ExecutionError::ExecutionFailed => 128,
            ExecutionError::FailedToWriteStdin => 129,
        }
    }
}

pub fn spawn_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>, local_env: Option<&Vec<(String, String)>>, capture_output: bool) -> Result<Child, ShellError> {
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

  // Inject the local environment variables if any
  if let Some(env_vars) = local_env {
      for (key, value) in env_vars {
          process.env(key, value);
      }
  }

  // Spawn the process
  let mut child = match process.spawn() {
      Ok(child) => child,
      Err(_error) => {
          return Err(ShellError::Execution(ExecutionError::CommandNotFound(program.to_string())));
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

pub fn execute_program(program: &str, args: &Vec<String>, input: &Option<Vec<u8>>, local_env: Option<&Vec<(String, String)>>, capture_output: bool) -> Result<CmdOutput, ShellError> {
  match spawn_program(program, args, input, local_env, capture_output) {
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
