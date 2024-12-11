use std::io::{Write};
use std::env;
use std::process;
use std::collections::VecDeque;

use crate::core::ShellError;
use crate::eval::execute;

pub fn match_builtin(stdout: &mut dyn Write, command: &str, args: &VecDeque<&str>) -> Result<Option<u8>, ShellError> {
    if command == "exit" {
        cmd_exit();
    }
    return match command {
        "cd" => Ok(Some(cmd_cd(&args))),
        "pwd" => Ok(Some(cmd_pwd())),
        "export" => {
            match cmd_export(stdout, args) {
                Ok(res) => return Ok(Some(res)),
                Err(error) => return Err(error)
            }
        },
        _ => Ok(None)
    }
}

fn cmd_exit() {
    process::exit(0x0000);
}

fn cmd_cd(args: &VecDeque<&str>) -> u8 {
    env::set_current_dir(args[0]).unwrap();
    return 0;
}

fn cmd_pwd() -> u8 {
    println!("{}", env::current_dir().unwrap().display());
    return 0;
}

fn cmd_export(stdout: &mut dyn Write, args: &VecDeque<&str>) -> Result<u8, ShellError> {
    if args.len() <= 0 {
        return execute(stdout,"env", &VecDeque::new());
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return Ok(0);
}