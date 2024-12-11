extern crate console;

use std::env;
use std::io::Write;
use console::Term;
use console::style;
use console::Key;

pub struct Shell {
    pub term: Term,
    pub status: u8,
    hisidx: Option<usize>,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            term: Term::stdout(),
            status: 0,
            history: Vec::new(),
            hisidx: None
        }
    }

    pub fn add_input(&mut self, c: char) {
        if self.hisidx.is_some() {
            self.history[self.hisidx.unwrap()].insert(self.cursor, c);
        } else {
            self.input.insert(self.cursor, c);
        }
        self.cursor += 1;
        self.update_input(true);
    }

    pub fn move_history(&mut self, up: bool) {
        match self.hisidx {
            Some(idx) => {
                if up && idx > 0 {
                    self.hisidx = Some(idx - 1);
                    self.update_input(false);
                } else if !up {
                    if idx < self.history.len() - 1 {
                        self.hisidx = Some(idx + 1);
                        self.update_input(false);
                    } else {
                        self.hisidx = None;
                        self.update_input(true);
                    }
                }
            },
            None => {
                if up && self.history.len() > 0 {
                    self.hisidx = Some(self.history.len()-1);
                    self.update_input(false);
                }
            }
        }
    }

    pub fn get_input(&self) -> &String {
        if self.hisidx.is_some() {
            return &self.history[self.hisidx.unwrap()];
        } else {
            return &self.input;
        }
    }

    pub fn historize_input(&mut self, vinput: &String) {
        let lastinput = self.history.last();
        if lastinput.is_some() {
            if *vinput != *lastinput.unwrap() {
                self.history.push(vinput.clone());
            }
        } else {
            self.history.push(vinput.clone());
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    pub fn update_input(&mut self, replace_cursor: bool) {
        self.term.clear_line().unwrap();
        let input = self.get_input();
        print!("{}", *input);
        std::io::stdout().flush().unwrap();
        if replace_cursor && input.len() > self.cursor {
            self.term.move_cursor_left(input.len() - self.cursor).unwrap();
        } else {
            self.cursor = input.len();
        }
    }

}