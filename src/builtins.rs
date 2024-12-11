use std::env;
use std::collections::VecDeque;

use crate::core::{ShellError, ShellState};
use crate::eval::execute;

pub fn match_builtin(state: &mut ShellState, command: &str, args: &VecDeque<&str>) -> Result<Option<u8>, ShellError> {
    return match command {
        "exit" => Ok(Some(cmd_exit(state))),
        "cd" => Ok(Some(cmd_cd(&args))),
        "pwd" => Ok(Some(cmd_pwd())),
        "export" => {
            match cmd_export(state, args) {
                Ok(res) => return Ok(Some(res)),
                Err(error) => return Err(error)
            }
        },
        _ => Ok(None)
    }
}

fn cmd_exit(state: &mut ShellState) -> u8 {
    state.running = false;
    return 0;
}

fn cmd_cd(args: &VecDeque<&str>) -> u8 {
    env::set_current_dir(args[0]).unwrap();
    env::set_var("PWD", env::current_dir().unwrap());
    return 0;
}

fn cmd_pwd() -> u8 {
    println!("{}", env::current_dir().unwrap().display());
    return 0;
}

fn cmd_export(state: &mut ShellState, args: &VecDeque<&str>) -> Result<u8, ShellError> {
    if args.len() <= 0 {
        return execute(state,"env", &VecDeque::new());
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return Ok(0);
}