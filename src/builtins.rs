use std::env;
use std::collections::VecDeque;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

use crate::cmdoutput::CmdOutput;
use crate::core::ShellError;
use crate::eval::{execute, ExecutionError};

pub fn match_builtin(command: &str, args: &VecDeque<&str>) -> Result<Option<CmdOutput>, ShellError> {
    match command {
        "exit" => cmd_exit(),
        "cd" => cmd_cd(&args),
        "pwd" => cmd_pwd(),
        "export" => cmd_export(args),
        _ => Ok(None)
    }
}

fn cmd_exit() -> Result<Option<CmdOutput>, ShellError> {
    return Err(ShellError::ExitRequest());
}

fn cmd_cd(args: &VecDeque<&str>) -> Result<Option<CmdOutput>, ShellError> {
    match args.len() {
        0 => {
            env::set_var("OLDPWD", env::current_dir().unwrap());
            env::set_current_dir(env::var("HOME").unwrap()).unwrap();
        }
        1 => {
            if args[0].len() == 1 && args[0].chars().next() == Some('-') {
                let oldpwd = env::current_dir().unwrap();
                env::set_current_dir(env::var("OLDPWD").unwrap()).unwrap();
                env::set_var("OLDPWD", oldpwd);
            } else {
                env::set_var("OLDPWD", env::current_dir().unwrap());
                env::set_current_dir(args[0]).unwrap();
            }
        },
        _ => {
            return Err(ShellError::Execution(ExecutionError::new(1, "too many arguments for cd".to_string())));
        }
    }
    env::set_var("PWD", env::current_dir().unwrap());
    return Ok(Some(CmdOutput::from_status(ExitStatus::from_raw(0))));
}

fn cmd_pwd() -> Result<Option<CmdOutput>, ShellError> {
    let mut output = CmdOutput::from_status(ExitStatus::from_raw(0));
    let mut env_output = env::current_dir().unwrap().to_string_lossy().as_bytes().to_vec();
    env_output.push(b'\n');
    output.stdout = Some(env_output);
    return Ok(Some(output));
}

fn cmd_export(args: &VecDeque<&str>) -> Result<Option<CmdOutput>, ShellError> {
    if args.len() <= 0 {
        return execute("env", &VecDeque::new());
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return Ok(Some(CmdOutput::from_status(ExitStatus::from_raw(0))));
}