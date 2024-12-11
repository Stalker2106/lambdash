use std::io::{Write, stdout};
extern crate crossterm;

use crossterm::{
    cursor, QueueableCommand, event::{
        read, Event, KeyCode
    }
};

mod core;
mod tokenizer;
mod eval;
mod builtins;
mod prompt;
mod history;

use core::ShellError;
use prompt::Prompt;
use history::History;
use eval::eval;

fn main() {
    let running = true;
    let mut stdout = stdout();
    let mut prompt = Prompt::new();
    let mut history = History::new();
    let mut status: u8 = 0;
    // main loop
    while running {
        let mut reading: bool = true;
        stdout.write(prompt.get_ps1().as_bytes());
        stdout.flush().unwrap();
        // read loop
        while reading {
            if let Ok(e) = read() {
                match e {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Char(c) => {
                                prompt.add_char(c);
                                stdout.queue(cursor::MoveRight(1)).unwrap();
                            }
                            KeyCode::Left => {
                                prompt.move_cursor(-1);
                                stdout.queue(cursor::MoveLeft(1)).unwrap();
                            },
                            KeyCode::Right => {
                                prompt.move_cursor(1);
                                stdout.queue(cursor::MoveLeft(1)).unwrap();
                            },
                            KeyCode::Up => (),
                            KeyCode::Down => (),
                            KeyCode::Tab => (),
                            KeyCode::Backspace => {
                                prompt.remove_char();
                                stdout.queue(cursor::MoveLeft(1)).unwrap();
                            },
                            KeyCode::Enter => {
                                reading = false;
                                stdout.queue(cursor::MoveToNextLine(1)).unwrap();
                            },
                            _ => ()
                        }
                        stdout.flush().unwrap();
                    }
                    _ => ()
                }
            }
        }
        if prompt.has_input() {
            let input: &String = prompt.get_input();
            history.submit_value(input);
            match eval(&mut stdout, input) {
                Ok(s) => {
                    if let Some(res) = s {
                        status = res;
                    }
                }
                Err(e) => {
                    match e {
                        ShellError::Execution(error) => stdout.write(format!("Error: {}\n", error.details).as_bytes()).unwrap(),
                        ShellError::Tokenization(error) => stdout.write(format!("Error: {}\n", error.details).as_bytes()).unwrap()
                    };
                }
            }
            prompt.clear_input();
        }
    }
}