use std::io::{stdout, stderr};
extern crate crossterm;

use crossterm::{
    cursor,
    event::{
        read,
        Event,
        KeyCode, KeyModifiers
    },
    style::{Color, Print, ResetColor, SetForegroundColor},
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

fn clear_prompt_input(state: &mut ShellState, prompt: &Prompt) {
    let p_cursor = prompt.get_cursor();
    if p_cursor > 0 {
        state.stdout.queue(cursor::MoveLeft(p_cursor as u16)).unwrap();
    }
    state.stdout.queue(Clear(ClearType::UntilNewLine)).unwrap();
}

fn main() {
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut state: ShellState = ShellState::new(&mut stdout, &mut stderr);
    let mut prompt = Prompt::new();
    let mut history = History::new();
    // main loop
    loop {
        prompt.print_ps1(&mut state);
        state.stdout.flush().unwrap();
        let mut history_idx: Option<usize> = None;
        // read loop
        crossterm::terminal::enable_raw_mode().unwrap();
        loop {
            if let Ok(e) = read() {
                match e {
                    Event::Key(event) => {
                        if event.modifiers.contains(KeyModifiers::CONTROL) {
                            match event.code {
                                KeyCode::Char(c) => {
                                    match c {
                                        'c' => {
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
                                            state.stdout.queue(Clear(ClearType::All)).unwrap()
                                                        .queue(cursor::MoveTo(0,0)).unwrap();
                                            prompt.print_ps1(&mut state);
                                            let input = &prompt.get_input();
                                            state.stdout.queue(Print(input)).unwrap();
                                            let diff = input.len() - prompt.get_cursor();
                                            if diff > 0 {
                                                state.stdout.queue(cursor::MoveLeft((diff) as u16)).unwrap();
                                            }
                                            state.stdout.flush().unwrap();
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
                                            history_idx = index;
                                            prompt.stash_input();
                                            clear_prompt_input(&mut state, &prompt);
                                            prompt.set_input(history.get(history_idx.unwrap()));
                                            state.stdout.queue(Print(prompt.get_input())).unwrap();
                                        }
                                    } else {
                                        if history_idx.unwrap() > 0 {
                                            history_idx = Some(history_idx.unwrap() - 1);
                                            clear_prompt_input(&mut state, &prompt);
                                            prompt.set_input(history.get(history_idx.unwrap()));
                                            state.stdout.queue(Print(prompt.get_input())).unwrap();
                                        }
                                    }
                                },
                                KeyCode::Down => {
                                    if history_idx.is_some() {
                                        if let Some(last_index) = history.get_first_index() {
                                            if history_idx.unwrap() < last_index {
                                                history_idx = Some(history_idx.unwrap() + 1);
                                                clear_prompt_input(&mut state, &prompt);
                                                prompt.set_input(history.get(history_idx.unwrap()));
                                                state.stdout.queue(Print(prompt.get_input())).unwrap();
                                            } else {
                                                history_idx = None;
                                                clear_prompt_input(&mut state, &prompt);
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
                                    prompt.append_char('\n');
                                    state.stdout.queue(Print("\n")).unwrap()
                                                .queue(cursor::MoveToNextLine(1)).unwrap();
                                    break;
                                },
                                _ => ()
                            }
                        }
                        state.stdout.flush().unwrap();
                    }
                    _ => ()
                }
            }
        }
        crossterm::terminal::disable_raw_mode().unwrap();
        if prompt.has_input() {
            let input: &String = prompt.get_input();
            history.submit(input);
            match eval_expr(&mut state, input) {
                Ok(res) => {
                    if let Some(output) = res {
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
                }
                Err(e) => {
                    match e {
                        ShellError::Execution(error) => {
                            state.status = error.status;
                            state.stdout.queue(SetForegroundColor(Color::Red)).unwrap()
                                        .queue(Print(format!("error: {}", error.details))).unwrap()
                                        .queue(ResetColor).unwrap()
                                        .queue(Print("\n")).unwrap();
                        },
                        ShellError::Tokenization(error) => {
                            state.status = error.status;
                            state.stdout.queue(SetForegroundColor(Color::Red)).unwrap()
                                        .queue(Print(format!("syntax error: {}", error.details))).unwrap()
                                        .queue(ResetColor).unwrap()
                                        .queue(Print("\n")).unwrap();
                        },
                        ShellError::ExitRequest() => break
                    };
                    state.stdout.queue(cursor::MoveToNextLine(1)).unwrap();
                }
            }
            prompt.clear_input();
        } else {
            state.stdout.queue(Print("exit\n")).unwrap();
            break;
        }
    }
}