use std::env;
use std::io::Cursor;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

use crossterm::style::Print;
use crossterm::QueueableCommand;

use crate::cmdoutput::CmdOutput;
use crate::core::{ShellState, ShellError};
use crate::eval::{execute, ExecutionError};

pub fn match_builtin(state: &mut ShellState, command: &str, args: &Vec<&str>, input: &Option<CmdOutput>) -> Result<Option<CmdOutput>, ShellError> {
    match command {
        "exit" => cmd_exit(),
        "alias" => cmd_alias(state, args),
        "cd" => cmd_cd(args),
        "pwd" => cmd_pwd(),
        "export" => cmd_export(args, input),
        _ => Ok(None)
    }
}

fn cmd_exit() -> Result<Option<CmdOutput>, ShellError> {
    return Err(ShellError::ExitRequest());
}

fn cmd_alias(state: &mut ShellState, args: &Vec<&str>) -> Result<Option<CmdOutput>, ShellError> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    match args.len() {
        0 => {
            for (name, command) in state.aliases.iter() {
                cursor.queue(Print(format!("alias {} {}\n", name, command))).unwrap();
            }
        },
        _ => {
            let combined_args = args.join(" ");
            if let Some(index) = combined_args.rfind('=') {
                let (alias, cmd) = combined_args.split_at(index);
                state.aliases.insert(alias.replace('=', " ").to_string(), cmd.to_string());
            } else {
                return Err(ShellError::Execution(ExecutionError::new(1, "alias: body cannot be empty".to_string())));
            }
        }
    }
    return Ok(Some(CmdOutput{status: ExitStatus::from_raw(0), stdout: Some(output), stderr: None }))
}

fn cmd_cd(args: &Vec<&str>) -> Result<Option<CmdOutput>, ShellError> {
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

fn cmd_export(args: &Vec<&str>, input: &Option<CmdOutput>) -> Result<Option<CmdOutput>, ShellError> {
    if args.len() <= 0 {
        return execute("env", &Vec::new(), input);
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return Ok(Some(CmdOutput::from_status(ExitStatus::from_raw(0))));
}