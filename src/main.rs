use std::io::{stdout, stderr};
use std::env;
extern crate crossterm;

use crossterm::{
    cursor,
    event::{
        read,
        Event,
        KeyCode, KeyModifiers
    },
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
    terminal::{
        Clear,
        ClearType
    },
    QueueableCommand
};

mod fsio;
mod core;
mod expression;
mod config;
mod tokenizer;
mod eval;
mod builtins;
mod prompt;
mod promptscript;
mod cmdoutput;
mod history;

use core::{ShellError, ShellState};
use prompt::{CursorMovement, Prompt};
use eval::eval_expr;
use promptscript::eval_ps;

fn clear_prompt_input(state: &mut ShellState) {
    let (ps1col, ps1row) = state.ps1pos;
    state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap()
                .queue(Clear(ClearType::FromCursorDown)).unwrap();
}

fn print_prompt_input(state: &mut ShellState, input: &String) {
    let (ps1col, ps1row) = &mut state.ps1pos;
    let (_, termrows) = terminal::size().unwrap();
    let input_lines: Vec<&str> = input.split('\n').collect();
    for (row, input_line) in input_lines.iter().enumerate() {
        state.stdout.queue(cursor::MoveToColumn(*ps1col)).unwrap();
        if row < input_lines.len() - 1 {
            state.stdout.queue(Print(format!("{}\n", input_line))).unwrap();
            // handle newline scrolling whole term, therefore moving ps1
            if *ps1row + (row as u16) >= termrows - 1 {
                *ps1row -= 1;
            }
        } else {
            state.stdout.queue(Print(format!("{}", input_line))).unwrap();
        }
    }
}

pub fn update_size(width: u16, height: u16) {
    env::set_var("COLUMNS", width.to_string());
    env::set_var("LINES", height.to_string());
}

pub fn prompt_readloop(state: &mut ShellState, prompt: &mut Prompt, history_idx: &mut Option<usize>) -> i32 {
    let mut chars_read = -1;
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        if let Ok(e) = read() {
            match e {
                Event::Resize(width, height) => update_size(width, height),
                Event::Key(event) => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        match event.code {
                            KeyCode::Char(c) => {
                                match c {
                                    'c' => {
                                        chars_read = 0;
                                        prompt.clear_input();
                                        state.stdout.queue(Print("^C\n")).unwrap()
                                                    .queue(cursor::MoveToColumn(0)).unwrap();
                                        break;
                                    },
                                    'd' => {
                                        if !prompt.has_input() {
                                            break;
                                        }
                                    }
                                    'l' => {
                                        chars_read = 0;
                                        state.stderr.queue(Clear(ClearType::All)).unwrap()
                                                    .queue(cursor::MoveTo(0,0)).unwrap();
                                        state.stdout.queue(cursor::MoveTo(0,0)).unwrap()
                                                    .queue(Clear(ClearType::All)).unwrap();
                                        prompt.stash_input();
                                        prompt.clear_input();
                                        break;
                                    },
                                    'k' => {
                                        if prompt.truncate_input() {
                                            state.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
                                        }
                                    }
                                    _ => ()
                                }
                            },
                            _ => ()
                        }
                    } else if event.modifiers.contains(KeyModifiers::ALT) {
                        match event.code {
                            KeyCode::Left => {
                                let diff = prompt.move_cursor_left(CursorMovement::Word);
                                if diff > 0 {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16,ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Right => {
                                let diff = prompt.move_cursor_right(CursorMovement::Word);
                                if diff > 0 {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                                }
                            },
                            _ => ()
                        }
                    } else {
                        match event.code {
                            KeyCode::Char(c) => {
                                prompt.add_char(c);
                                chars_read += 1;
                                state.stdout.queue(Print(c)).unwrap();
                                clear_prompt_input(state);
                                print_prompt_input(state, prompt.get_input());
                                let (ps1col, ps1row) = state.ps1pos;
                                let (curcol, currow) = prompt.get_cursor_offset();
                                state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                            }
                            KeyCode::Home => {
                                if prompt.move_cursor(prompt::CursorPosition::Origin) {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap();
                                }
                            },
                            KeyCode::End => {
                                if prompt.move_cursor(prompt::CursorPosition::End) {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Left => {
                                let diff = prompt.move_cursor_left(CursorMovement::One);
                                if diff > 0 {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16,ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Right => {
                                let diff = prompt.move_cursor_right(CursorMovement::One);
                                if diff > 0 {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Up => {
                                if !history_idx.is_some() {
                                    let index = state.history.get_first_index();
                                    if index.is_some() {
                                        *history_idx = index;
                                        prompt.stash_input();
                                        clear_prompt_input(state);
                                        prompt.set_input(state.history.get(history_idx.unwrap()));
                                        print_prompt_input(state, prompt.get_input());
                                    }
                                } else {
                                    if history_idx.unwrap() > 0 {
                                        *history_idx = Some(history_idx.unwrap() - 1);
                                        clear_prompt_input(state);
                                        prompt.set_input(state.history.get(history_idx.unwrap()));
                                        print_prompt_input(state, prompt.get_input());
                                    }
                                }
                            },
                            KeyCode::Down => {
                                if history_idx.is_some() {
                                    if let Some(last_index) = state.history.get_first_index() {
                                        if history_idx.unwrap() < last_index {
                                            *history_idx = Some(history_idx.unwrap() + 1);
                                            clear_prompt_input(state);
                                            prompt.set_input(state.history.get(history_idx.unwrap()));
                                            print_prompt_input(state, prompt.get_input());
                                        } else {
                                            *history_idx = None;
                                            clear_prompt_input(state);
                                            prompt.unstash_input();
                                            print_prompt_input(state, prompt.get_input());
                                        }
                                    }
                                }
                            },
                            KeyCode::Tab => (),
                            KeyCode::Delete => {
                                if prompt.remove_char(false) {
                                    clear_prompt_input(state);
                                    print_prompt_input(state, prompt.get_input());
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Backspace => {
                                if prompt.remove_char(true) {
                                    clear_prompt_input(state);
                                    print_prompt_input(state, prompt.get_input());
                                    let (ps1col, ps1row) = state.ps1pos;
                                    let (curcol, currow) = prompt.get_cursor_offset();
                                    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
                                }
                            },
                            KeyCode::Enter => {
                                chars_read = 0;
                                state.stdout.queue(Print("\n")).unwrap()
                                            .queue(cursor::MoveToColumn(0)).unwrap();
                                break;
                            },
                            _ => ()
                        }
                    }
                }
                _ => ()
            }
        }
        state.stdout.flush().unwrap();
    }
    crossterm::terminal::disable_raw_mode().unwrap();
    return chars_read;
}

fn main() {
    if let Ok((width, height)) = terminal::size() {
        update_size(width, height);
    }
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut state: ShellState = ShellState::new(&mut stdout, &mut stderr);
    let mut prompt = Prompt::new();
    let ps1script = state.config.prompt.ps1.clone();
    // main loop
    loop {
        prompt.unstash_input();
        let ps1out = eval_ps(&mut state, &ps1script);
        if let Ok(ps1) = String::from_utf8(ps1out.stdout) {
            state.stdout.queue(Print(ps1)).unwrap();
        }
        state.ps1pos = cursor::position().unwrap();
        state.stderr.flush().unwrap();
        state.stdout.queue(Print(prompt.get_input())).unwrap();
        state.stdout.flush().unwrap();
        let mut history_idx: Option<usize> = None;
        // read loop
        let mut chars_read = prompt_readloop(&mut state, &mut prompt, &mut history_idx);
        if prompt.has_input() {
            let mut expr = prompt.get_input().clone();
            // eval loop
            loop {
                match eval_expr(&mut state, &expr) {
                    Ok(_) => {
                        state.history.submit(&expr);
                        break; // Exit loop after successful execution
                    }
                    Err(e) => match e {
                        ShellError::Execution(error) => {
                            state.history.submit(&expr);
                            state.status = error.status;
                            state.stdout
                                .queue(SetForegroundColor(Color::Red)).unwrap()
                                .queue(Print(format!("{}", error.details))).unwrap()
                                .queue(ResetColor).unwrap()
                                .queue(Print("\n")).unwrap();
                            break; // Exit loop on execution error
                        }
                        ShellError::Tokenization(_) => {
                            prompt.add_char('\n');
                            state.ps1pos.1 -= 1;
                            let (ps1col, ps1row) = state.ps1pos;
                            let (curcol, currow) = prompt.get_cursor_offset();
                            state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap()
                                        .flush().unwrap();
                            prompt_readloop(&mut state, &mut prompt, &mut history_idx);
                            expr = prompt.get_input().clone();
                        }
                        ShellError::ExitRequest => {
                            chars_read = -1;
                            break;
                        }
                        _ => ()
                    },
                }
            }
            prompt.clear_input();
        }
        if chars_read == -1 {
            break;
        }
    }
}