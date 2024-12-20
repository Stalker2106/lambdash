use std::env;
use std::io::Write;
use std::collections::HashMap;
use std::process::Child;

use crossterm::terminal;

use crate::core::config::{ShellConfig, load};
use crate::features::history::History;

pub struct ShellState<'a> {
    pub status: i32,
    pub ps1pos: (u16, u16),
    pub termsize: (u16, u16),
    pub jobs: Vec<Child>,
    pub history: History,
    pub aliases: HashMap<String, String>,
    pub config: ShellConfig,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write
}

impl<'a> ShellState<'a> {
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> ShellState<'a> {
        ShellState {
            status: 0,
            ps1pos: (0,0),
            termsize: terminal::size().expect("unable to obtain terminal size."),
            jobs: Vec::new(),
            history: History::load(),
            aliases: HashMap::new(),
            config: load(),
            stdout: out,
            stderr: err,
        }
    }

    pub fn update_size(&mut self, width: u16, height: u16) {
        self.termsize = (width, height);
        env::set_var("COLUMNS", width.to_string());
        env::set_var("LINES", height.to_string());
      }
}