use std::{env, fs};

use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;

use crate::core::core::{ShellError, ShellState};
use crate::rendering::columns::render_columns;

struct AutocompleteState {
    index: Option<usize>,
    items: Vec<String>
}

pub struct Autocomplete {
    state: Option<AutocompleteState>
}

impl Autocomplete {
    pub fn new() -> Autocomplete {
        return Autocomplete{
            state: None
        };
    }
    pub fn complete(&mut self, state: &mut ShellState, input: &str) -> Result<Option<String>, ShellError> {
        let mut res: Option<String> = None;
        if let Some(astate) = self.state.as_mut() {
            if let Some(index) = astate.index.as_mut() {
                if *index < astate.items.len() - 1 {
                    *index += 1;
                } else {
                    *index = 0;
                }
                res = astate.items.get(*index).cloned();
            } else {
                astate.index = Some(0);
                res = astate.items.get(0).cloned();
            }
        } else {
            let mut res: Vec<String> = Vec::new();
            if input.starts_with("cd") || input.contains('/') || input.contains('.') {
                res = path_completion(&input);
            } else {
                res = command_completion(&input);
            }
            match res.len() {
                0 => {
                    return Ok(None);
                },
                1 => {
                    return Ok(res.last().cloned());
                },
                _ => {
                    let astate = AutocompleteState {
                        index: None,
                        items: res
                    };
                    self.state = Some(astate);
                }
            }
        }
        if let Some(astate) = &self.state {
            print_options(state, &astate).unwrap();
        }
        return Ok(res);
    }

    pub fn reset(&mut self, state: &mut ShellState) {
        self.state = None;
        state.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
    }
}

fn command_completion(input: &str) -> Vec<String> {

    let mut available: Vec<String> = Vec::new();
    let mut searchpaths: Vec<String> = Vec::new();
    if let Ok(path) = env::var("PATH") {
        searchpaths = path.split(":").map(String::from).collect();
    }
    for searchpath in searchpaths {
        if let Ok(entries) = fs::read_dir(searchpath) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(path_str) = path.file_name().and_then(|f| f.to_str()) {
                            // Filter commands based on input
                            if path_str.starts_with(input) {
                                available.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    available.sort();
    return available;
}

fn print_options(state: &mut ShellState, astate: &AutocompleteState) -> Result<(), ShellError>  {
    match render_columns(state, &astate.items, 0, state.termsize.1 / 2) {
        Ok(out) => {
            if let Ok(output) = String::from_utf8(out.stdout) {
                state.stdout.queue(Print(output)).unwrap();
                return Ok(());
            } else {
                return Err(ShellError::ExitRequest);
            }
        },
        Err(error) => {
            return Err(error);
        }
    }
}

fn path_completion(input: &str) -> Vec<String> {
    let mut available = Vec::new();
    let (dir_path, prefix) = if let Some(pos) = input.rfind('/') {
        input.split_at(pos + 1)
    } else {
        ("./", input)
    };
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with(prefix) {
                            available.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    available.sort();
    return available;
}

fn custom_completion(input: &str) {

}