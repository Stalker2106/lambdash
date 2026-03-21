use core::{error::ShellError, readloop::prompt_readloop};
use std::io::{stdout, stderr};
extern crate crossterm;

#[cfg(windows)]
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
#[cfg(windows)]
use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_WINDOW_INPUT};
#[cfg(windows)]
use winapi::um::wincon::FlushConsoleInputBuffer;
#[cfg(windows)]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::STD_INPUT_HANDLE;

#[cfg(windows)]
fn restore_console_mode() {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == INVALID_HANDLE_VALUE {
            return;
        }

        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return;
        }

        let desired = ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_WINDOW_INPUT;
        let _ = SetConsoleMode(handle, desired);
        let _ = FlushConsoleInputBuffer(handle);
    }
}

#[cfg(not(windows))]
fn restore_console_mode() {}

use features::autocomplete::Autocomplete;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    QueueableCommand
};
use rendering::prompt::{align_cursor_with_prompt, print_prompt};

mod core;
mod eval;
mod features;
mod parser;
mod rendering;

use core::core::{ShellState};
use features::prompt::Prompt;
use eval::jobs::poll_jobs;
use eval::eval::eval_expr;

fn main() {
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut state: ShellState = ShellState::new(&mut stdout, &mut stderr);
    let mut prompt = Prompt::new(&state.config.prompt.ps1);
    // main loop
    loop {
        poll_jobs(&mut state);
        let mut autocomplete = Autocomplete::new();
        let mut history_idx: Option<usize> = None;
        prompt.unstash_input();
        print_prompt(&mut state, &prompt);
        state.stdout.flush().unwrap();
        state.ps1pos = cursor::position().unwrap();
        state.stderr.flush().unwrap();
        // read loop
        let mut chars_read = prompt_readloop(&mut state, &mut autocomplete, &mut prompt, &mut history_idx);
        state.stdout.queue(Print("\n")).unwrap()
                    .queue(cursor::MoveToColumn(0)).unwrap();
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
                        error => {
                            state.history.submit(&expr);
                            state.status = error.status() as i32;
                            if let Ok(error_str) = String::from_utf8(error.to_output(prompt.get_input())) {
                                state.stdout
                                    .queue(SetForegroundColor(Color::Red)).unwrap()
                                    .queue(Print(error_str)).unwrap()
                                    .queue(ResetColor).unwrap()
                                    .queue(Print("\n")).unwrap();
                            }
                            break; // Exit loop on execution error
                        }
                    },
                }
            }
            // Force restore of terminal in case of interactive child processes leaving it in raw mode
            crossterm::terminal::disable_raw_mode().unwrap();
            restore_console_mode();

            let (width, height) = crossterm::terminal::size().unwrap();
            state.update_size(width, height);
            state.ps1pos = cursor::position().unwrap();
            prompt.clear_input();
        }
        if chars_read == -1 {
            state.history.persist();
            break;
        }
    }
}