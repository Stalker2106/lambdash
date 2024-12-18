use std::io::{stdout, stderr};
use std::env;
extern crate crossterm;

use features::autocomplete::Autocomplete;
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

mod core;
mod eval;
mod features;
mod parser;
mod rendering;

use core::core::{ShellError, ShellState};
use features::prompt::{CursorMovement, CursorPosition, Prompt};
use eval::eval::eval_expr;
use features::promptscript::eval_ps;

fn clear_prompt(state: &mut ShellState, prompt: &Prompt) {
    let (_, ps1row) = state.ps1pos;
    for line in 0..prompt.get_input_rows() {
        state.stdout.queue(cursor::MoveTo(0, ps1row + line as u16)).unwrap()
                    .queue(Clear(ClearType::CurrentLine)).unwrap();
    }
    state.stdout.queue(cursor::MoveTo(0, ps1row)).unwrap();
}

fn clear_prompt_input(state: &mut ShellState, prompt: &Prompt) {
    let (ps1col, ps1row) = state.ps1pos;
    for line in 0..prompt.get_input_rows() {
        state.stdout.queue(cursor::MoveTo(ps1col, ps1row + line as u16)).unwrap()
                    .queue(Clear(ClearType::UntilNewLine)).unwrap();
    }
    state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap();
}

fn print_prompt_input(state: &mut ShellState, input: &String) {
    let (ps1col, ps1row) = &mut state.ps1pos;
    let (_, termrows) = terminal::size().unwrap();
    let input_lines: Vec<&str> = input.split('\n').collect();
    for (row, input_line) in input_lines.iter().enumerate() {
        state.stdout.queue(cursor::MoveTo(*ps1col, *ps1row)).unwrap();
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

fn align_cursor_with_prompt(state: &mut ShellState, prompt: &Prompt) {
    let (ps1col, ps1row) = state.ps1pos;
    let (curcol, currow) = prompt.get_cursor_offset();
    state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
}

pub fn update_size(state: &mut ShellState, width: u16, height: u16) {
    state.termsize = (width, height);
    env::set_var("COLUMNS", width.to_string());
    env::set_var("LINES", height.to_string());
}

pub fn prompt_readloop(state: &mut ShellState, autocomplete: &mut Autocomplete, prompt: &mut Prompt, history_idx: &mut Option<usize>) -> i32 {
    let mut chars_read = -1;
    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        if let Ok(e) = read() {
            match e {
                Event::Resize(width, height) => update_size(state, width, height),
                Event::Key(event) => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        match event.code {
                            KeyCode::Char(c) => {
                                match c {
                                    'c' => {
                                        chars_read = 0;
                                        prompt.clear_input();
                                        autocomplete.reset(state);
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
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            KeyCode::Right => {
                                let diff = prompt.move_cursor_right(CursorMovement::Word);
                                if diff > 0 {
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            _ => ()
                        }
                    } else {
                        match event.code {
                            KeyCode::Char(c) => {
                                prompt.add_char(c);
                                chars_read += 1;
                                autocomplete.reset(state);
                                clear_prompt_input(state, &prompt);
                                print_prompt_input(state, prompt.get_input());
                                align_cursor_with_prompt(state, prompt);
                            }
                            KeyCode::Home => {
                                if prompt.move_cursor(CursorPosition::Origin) {
                                    let (ps1col, ps1row) = state.ps1pos;
                                    state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap();
                                }
                            },
                            KeyCode::End => {
                                if prompt.move_cursor(CursorPosition::End) {
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            KeyCode::Left => {
                                let diff = prompt.move_cursor_left(CursorMovement::One);
                                if diff > 0 {
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            KeyCode::Right => {
                                let diff = prompt.move_cursor_right(CursorMovement::One);
                                if diff > 0 {
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            KeyCode::Up => {
                                if !history_idx.is_some() {
                                    let index = state.history.get_first_index();
                                    if index.is_some() {
                                        *history_idx = index;
                                        prompt.stash_input();
                                        clear_prompt_input(state, &prompt);
                                        prompt.set_input(state.history.get(history_idx.unwrap()));
                                        print_prompt_input(state, prompt.get_input());
                                    }
                                } else {
                                    if history_idx.unwrap() > 0 {
                                        *history_idx = Some(history_idx.unwrap() - 1);
                                        clear_prompt_input(state, &prompt);
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
                                            clear_prompt_input(state, &prompt);
                                            prompt.set_input(state.history.get(history_idx.unwrap()));
                                            print_prompt_input(state, prompt.get_input());
                                        } else {
                                            *history_idx = None;
                                            clear_prompt_input(state, &prompt);
                                            prompt.unstash_input();
                                            print_prompt_input(state, prompt.get_input());
                                        }
                                    }
                                }
                            },
                            KeyCode::Tab => {
                                match autocomplete.complete(state, &prompt.get_input()) {
                                    Ok(res) => {
                                        if let Some(completed) = res {
                                            prompt.set_input(&completed);
                                            clear_prompt_input(state, &prompt);
                                            print_prompt_input(state, prompt.get_input());
                                        } else {
                                            align_cursor_with_prompt(state, prompt);
                                        }
                                    },
                                    Err(_) => ()
                                }
                            },
                            KeyCode::Delete => {
                                if prompt.remove_char(false) {
                                    clear_prompt_input(state, &prompt);
                                    print_prompt_input(state, prompt.get_input());
                                    align_cursor_with_prompt(state, prompt);
                                }
                            },
                            KeyCode::Backspace => {
                                if prompt.remove_char(true) {
                                    autocomplete.reset(state);
                                    clear_prompt_input(state, &prompt);
                                    print_prompt_input(state, prompt.get_input());
                                    align_cursor_with_prompt(state, prompt);
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

fn print_prompt(state: &mut ShellState, prompt: &Prompt) {
    let ps1out = eval_ps(state, &prompt.ps1);
    if let Ok(ps1) = String::from_utf8(ps1out.stdout) {
        state.stdout.queue(Print(ps1)).unwrap();
    }
}

fn main() {
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut state: ShellState = ShellState::new(&mut stdout, &mut stderr);
    let mut prompt = Prompt::new(&state.config.prompt.ps1);
    // main loop
    loop {
        let mut autocomplete = Autocomplete::new();
        let mut history_idx: Option<usize> = None;
        prompt.unstash_input();
        print_prompt(&mut state, &prompt);
        state.ps1pos = cursor::position().unwrap();
        state.stderr.flush().unwrap();
        state.stdout.queue(Print(prompt.get_input())).unwrap();
        state.stdout.flush().unwrap();
        // read loop
        let mut chars_read = prompt_readloop(&mut state, &mut autocomplete, &mut prompt, &mut history_idx);
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
                            align_cursor_with_prompt(&mut state, &prompt);
                            state.stdout.flush().unwrap();
                            prompt_readloop(&mut state, &mut autocomplete, &mut prompt, &mut history_idx);
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
            state.history.persist();
            break;
        }
    }
}