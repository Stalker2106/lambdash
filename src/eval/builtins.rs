use std::env;
use std::io::Cursor;

use crossterm::style::Print;
use crate::core::error::{ShellError, StatusEnum};
use crate::crossterm::QueueableCommand;

use crate::core::cmdoutput::CmdOutput;
use crate::core::core::ShellState;
use crate::eval::execute::execute_program;

#[derive(Debug)]
pub struct BuiltinError {
    pub status: u16,
    pub message: String
}

impl BuiltinError {
    pub fn new(code: u16, msg: String) -> BuiltinError {
        return BuiltinError {
            status: code,
            message: msg
        };
    }
}

impl StatusEnum for BuiltinError {
    fn status(&self) -> u16 {
        return self.status;
    }
}


pub fn match_builtin(state: &mut ShellState, command: &str, args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<CmdOutput, ShellError> {
    match command {
        "alias" => cmd_alias(state, args),
        "cd" => cmd_cd(args),
        "exit" => cmd_exit(),
        "export" => cmd_export(args, input),
        "history" => cmd_history(state, args),
        "pwd" => cmd_pwd(),
        "fg" => cmd_fg(state, args),
        _ => Err(ShellError::NoBuiltin)
    }
}

fn cmd_exit() -> Result<CmdOutput, ShellError> {
    return Err(ShellError::ExitRequest);
}

fn cmd_alias(state: &mut ShellState, args: &Vec<String>) -> Result<CmdOutput, ShellError> {
    let mut output = CmdOutput::new();
    let mut cursor = Cursor::new(&mut output.stdout);
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
                return Err(ShellError::Builtin(BuiltinError::new(1, "alias: body cannot be empty".to_string())));
            }
        }
    }
    output.status = Some(0);
    return Ok(output);
}

fn cmd_cd(args: &Vec<String>) -> Result<CmdOutput, ShellError> {
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
                env::set_current_dir(&args[0]).unwrap();
            }
        },
        _ => {
            return Err(ShellError::Builtin(BuiltinError::new(1, "too many arguments for cd".to_string())));
        }
    }
    env::set_var("PWD", env::current_dir().unwrap());
    return Ok(CmdOutput::from_status(0));
}

fn cmd_history(state: &ShellState, args: &Vec<String>) -> Result<CmdOutput, ShellError> {
    let mut output = CmdOutput::new();
    let history_values = state.history.get_values().clone();
    match args.len() {
        0 => {
            output.stdout = history_values.join("\n").into_bytes();
            output.status = Some(0);
            return Ok(output);
        }
        _ => {
            output.stdout = history_values.into_iter()
                                        .filter(|value| args.contains(value))
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                        .into_bytes();
            output.status = Some(0);
            Ok(output)
        }
    }
}

fn cmd_pwd() -> Result<CmdOutput, ShellError> {
    let mut output = CmdOutput::new();
    let mut env_output = env::current_dir().unwrap().to_string_lossy().as_bytes().to_vec();
    env_output.push(b'\n');
    output.stdout = env_output;
    output.status = Some(0);
    return Ok(output);
}

fn cmd_export(args: &Vec<String>, input: &Option<Vec<u8>>) -> Result<CmdOutput, ShellError> {
    if args.len() <= 0 {
        return execute_program("env", &Vec::new(), input, true);
    }
    for arg in args {
        let kv = arg.split('=').collect::<Vec<&str>>();
        env::set_var(kv[0].to_string(), kv[1].to_string());
    }
    return Ok(CmdOutput::from_status(0));
}

fn cmd_fg(state: &mut ShellState, args: &[String]) -> Result<CmdOutput, ShellError> {
    // Parse job id from args, default to last job
    let job_idx = if let Some(arg) = args.first() {
        let n: usize = arg.trim_start_matches('%').parse().unwrap_or(1);
        state.jobs.iter().position(|(id, _, _)| *id == n)
    } else {
        if state.jobs.is_empty() { None } else { Some(state.jobs.len() - 1) }
    };

    let Some(idx) = job_idx else {
        return Err(ShellError::Builtin(BuiltinError::new(1, "fg: no such job".to_string())));
    };

    let (_job_id, name, child) = &mut state.jobs[idx];

    // Resume if stopped (Unix only)
    #[cfg(unix)] {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGCONT);
    }

    state.stdout.queue(Print(format!("{}\n", name))).unwrap();
    state.stdout.flush().unwrap();

    // Wait for it to finish
    let _ = child.wait();
    state.jobs.remove(idx);

    Ok(CmdOutput::from_status(0))
}