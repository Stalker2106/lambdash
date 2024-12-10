extern crate console;

use std::env;
use std::io::Write;
use console::Term;
use console::style;
use console::Key;

pub struct Shell {
    pub term: Term,
    pub status: u8,
    pub input: String,
    pub cursor: u8,
    history: Vec<String>,
    hisidx: Option<usize>,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            term: Term::stdout(),
            status: 0,
            input: String::new(),
            cursor: 0,
            history: Vec::new(),
            hisidx: None
        }
    }

    pub fn add_input(&mut self, c: char) {
        if self.hisidx.is_some() {
            self.history[self.hisidx.unwrap()].push(c);
        } else {
            self.input.push(c);
        }
        print!("{}", c);
        std::io::stdout().flush().unwrap();
    }

    pub fn move_history(&mut self, up: bool) {
        match self.hisidx {
            Some(idx) => {
                if up && idx > 0 {
                    self.hisidx = Some(idx - 1);
                    self.update_input();
                } else if !up {
                    if idx > 0 {
                        self.hisidx = Some(idx + 1);
                        self.update_input();
                    } else {
                        self.hisidx = None;
                        self.update_input();
                    }
                }
            },
            None => {
                if up && self.history.len() > 0 {
                    self.hisidx = Some(self.history.len()-1);
                    self.update_input();
                }
            }
        }
    }

    pub fn historize_input(&mut self) {
        if let Some(lastinput) = self.history.last() {
            if self.input != *lastinput {
                self.history.push(self.input.clone());
            }
        } else {
            self.history.push(self.input.clone());
        }
    }

    pub fn print_prompt(&self) {
        let mut ps1 = String::new();
        ps1.push_str("λsh ");
        ps1.push_str(&format!("{}", env::current_dir().unwrap().display()));
        if self.status != 0 {
            ps1.push_str(&format!("({})", self.status));
        }
        ps1.push_str("> ");
        print!("{}", style(ps1).yellow());
        std::io::stdout().flush().unwrap();
    }
    
    pub fn update_input(&self) {
        self.term.clear_line().unwrap();
        self.print_prompt();
        if self.hisidx.is_some() {
            print!("{}", self.history[self.hisidx.unwrap()]);
        } else {
            print!("{}", self.input);
        }
        std::io::stdout().flush().unwrap();
    }

    pub fn poll_input(&mut self) {
        let mut reading = true;
        while reading {
            if let Ok(k) = self.term.read_key() {
                match k {
                    Key::Char(c) => {
                        self.add_input(c);
                    },
                    Key::ArrowLeft => println!("left"),
                    Key::ArrowRight => println!("right"),
                    Key::ArrowUp | Key::PageUp => {
                        self.move_history(true);
                    },
                    Key::ArrowDown | Key::PageDown => {
                        self.move_history(false);
                    },
                    Key::Backspace => {
                        self.input.pop();
                        self.update_input();
                    },
                    Key::Enter => {
                        println!("");
                        reading = false;
                    },
                    Key::Tab => {
                        println!("T");
                    },
                    Key::UnknownEscSeq(seq) => println!("{}", seq.iter().collect::<String>()),
                    _ => println!("any")
                }
            }
        }
    }
}