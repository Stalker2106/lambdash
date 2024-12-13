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

mod core;
mod config;
mod tokenizer;
mod eval;
mod builtins;
mod prompt;
mod promptscript;
mod cmdoutput;
mod history;

use core::{ShellError, ShellState};
use prompt::Prompt;
use history::History;
use eval::eval_expr;
use promptscript::eval_ps;

fn clear_prompt_input(state: &mut ShellState, prompt: &Prompt) {
    let p_cursor = prompt.get_cursor();
    if p_cursor > 0 {
        state.stdout.queue(cursor::MoveLeft(p_cursor as u16)).unwrap();
    }
    state.stdout.queue(Clear(ClearType::UntilNewLine)).unwrap();
}

pub fn update_size(width: u16, height: u16) {
    env::set_var("COLUMNS", width.to_string());
    env::set_var("LINES", height.to_string());
}

pub fn prompt_readloop(state: &mut ShellState, prompt: &mut Prompt, history: &History, history_idx: &mut Option<usize>) -> i32 {
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
                                                    .queue(cursor::MoveToNextLine(1)).unwrap();
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
                                    }
                                    _ => ()
                                }
                            }
                            _ => ()
                        }
                    } else {
                        match event.code {
                            KeyCode::Char(c) => {
                                prompt.add_char(c);
                                chars_read += 1;
                                let p_cursor = prompt.get_cursor();
                                state.stdout.queue(Print(c)).unwrap();
                                // if we are inserting, we need to print rest of buffer to preserve it
                                if p_cursor < prompt.get_input().len() {
                                    let rest = &prompt.get_input()[p_cursor..];
                                    state.stdout.queue(Print(rest)).unwrap()
                                                .queue(cursor::MoveLeft(rest.len() as u16)).unwrap();
                                }
                            }
                            KeyCode::Left => {
                                if let Some(moved) = prompt.move_cursor_left(1) {
                                    state.stdout.queue(cursor::MoveLeft(moved as u16)).unwrap();
                                }
                            },
                            KeyCode::Right => {
                                if let Some(moved) = prompt.move_cursor_right(1) {
                                    state.stdout.queue(cursor::MoveRight(moved as u16)).unwrap();
                                }
                            },
                            KeyCode::Up => {
                                if !history_idx.is_some() {
                                    let index = history.get_first_index();
                                    if index.is_some() {
                                        *history_idx = index;
                                        prompt.stash_input();
                                        clear_prompt_input(state, &prompt);
                                        prompt.set_input(history.get(history_idx.unwrap()));
                                        state.stdout.queue(Print(prompt.get_input())).unwrap();
                                    }
                                } else {
                                    if history_idx.unwrap() > 0 {
                                        *history_idx = Some(history_idx.unwrap() - 1);
                                        clear_prompt_input(state, &prompt);
                                        prompt.set_input(history.get(history_idx.unwrap()));
                                        state.stdout.queue(Print(prompt.get_input())).unwrap();
                                    }
                                }
                            },
                            KeyCode::Down => {
                                if history_idx.is_some() {
                                    if let Some(last_index) = history.get_first_index() {
                                        if history_idx.unwrap() < last_index {
                                            *history_idx = Some(history_idx.unwrap() + 1);
                                            clear_prompt_input(state, &prompt);
                                            prompt.set_input(history.get(history_idx.unwrap()));
                                            state.stdout.queue(Print(prompt.get_input())).unwrap();
                                        } else {
                                            *history_idx = None;
                                            clear_prompt_input(state, &prompt);
                                            prompt.unstash_input();
                                            state.stdout.queue(Print(prompt.get_input())).unwrap();
                                        }
                                    }
                                }
                            },
                            KeyCode::Tab => (),
                            KeyCode::Backspace => {
                                if prompt.remove_char() {
                                    state.stdout.queue(cursor::MoveLeft(1)).unwrap()
                                                .queue(Clear(ClearType::UntilNewLine)).unwrap();
                                    let p_cursor = prompt.get_cursor();
                                    let rest = &prompt.get_input()[p_cursor..];
                                    state.stdout.queue(Print(rest)).unwrap();
                                    if rest.len() > 0 {
                                        state.stdout.queue(cursor::MoveLeft(rest.len() as u16)).unwrap();
                                    }
                                }
                            },
                            KeyCode::Enter => {
                                chars_read = 0;
                                state.stdout.queue(Print("\n")).unwrap();
                                state.stdout.queue(cursor::MoveToColumn(0)).unwrap();
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
    let mut history = History::new();
    let ps1script = state.config.prompt.ps1.clone();
    // main loop
    loop {
        prompt.unstash_input();
        let ps1out = eval_ps(&mut state, &ps1script);
        if let Some(ps1_stdout) = ps1out.stdout {
            if let Ok(ps1) = String::from_utf8(ps1_stdout) {
                state.stdout.queue(Print(ps1)).unwrap();
            }
        }
        let (ps1column, _) = cursor::position().unwrap();
        state.stderr.flush().unwrap();
        state.stdout.queue(Print(prompt.get_input())).unwrap();
        state.stdout.flush().unwrap();
        let mut history_idx: Option<usize> = None;
        // read loop
        let mut chars_read = prompt_readloop(&mut state, &mut prompt, &history, &mut history_idx);
        if prompt.has_input() {
            let mut expr = prompt.get_input().clone();
            // eval loop
            loop {
                match eval_expr(&mut state, &expr) {
                    Ok(res) => {
                        if let Some(output) = res {
                            history.submit(&expr);
                            state.status = output.status;
                            if let Some(cmd_stdout) = output.stdout {
                                if let Ok(cmd_out) = String::from_utf8(cmd_stdout) {
                                    state.stdout.queue(Print(cmd_out)).unwrap();
                                }
                            }
                            if let Some(cmd_stderr) = output.stderr {
                                if let Ok(cmd_err) = String::from_utf8(cmd_stderr) {
                                    state.stderr.queue(Print(cmd_err)).unwrap();
                                }
                            }
                        }
                        break; // Exit loop after successful execution
                    }
                    Err(e) => match e {
                        ShellError::Execution(error) => {
                            history.submit(&expr);
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
                            state.stdout.queue(cursor::MoveToColumn(ps1column as u16)).unwrap()
                                        .flush().unwrap();
                            prompt_readloop(&mut state, &mut prompt, &history, &mut history_idx);
                            expr = prompt.get_input().clone();
                        }
                        ShellError::ExitRequest() => {
                            chars_read = -1;
                            break;
                        }
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