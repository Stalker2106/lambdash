
use std::env;
use std::process;
use std::collections::VecDeque;

use crate::executor::execute;

pub fn match_builtin(command: &str, args: &VecDeque<&str>) -> Option<u8> {
    if command == "exit" {
        cmd_exit();
    }
    return match command {
        "cd" => Some(cmd_cd(&args)),
        "pwd" => Some(cmd_pwd()),
        "export" => Some(cmd_export(args)),
        _ => None
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

fn cmd_export(args: &VecDeque<&str>) -> u8 {
    if args.len() <= 0 {
        return execute("env", &VecDeque::new()).unwrap();
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return 0;
}