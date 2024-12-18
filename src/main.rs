use core::readloop::prompt_readloop;
use std::io::{stdout, stderr};
extern crate crossterm;

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

use core::core::{ShellError, ShellState};
use features::prompt::Prompt;
use eval::eval::eval_expr;

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