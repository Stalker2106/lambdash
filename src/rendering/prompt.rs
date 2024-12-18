use crossterm::{cursor, style::Print, terminal::{self, Clear, ClearType}, QueueableCommand};

use crate::{core::core::ShellState, features::{prompt::Prompt, promptscript::eval_ps}};


pub fn clear_prompt_input(state: &mut ShellState, prompt: &Prompt) {
  let (ps1col, ps1row) = state.ps1pos;
  for line in 0..prompt.get_input_rows() {
      state.stdout.queue(cursor::MoveTo(ps1col, ps1row + line as u16)).unwrap()
                  .queue(Clear(ClearType::UntilNewLine)).unwrap();
  }
  state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap();
}

pub fn print_prompt_input(state: &mut ShellState, input: &String) {
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

pub fn align_cursor_with_prompt(state: &mut ShellState, prompt: &Prompt) {
  let (ps1col, ps1row) = state.ps1pos;
  let (curcol, currow) = prompt.get_cursor_offset();
  state.stdout.queue(cursor::MoveTo(ps1col + curcol as u16, ps1row + currow as u16)).unwrap();
}

pub fn print_prompt(state: &mut ShellState, prompt: &Prompt) {
  let ps1out = eval_ps(state, &prompt.ps1);
  if let Ok(ps1) = String::from_utf8(ps1out.stdout) {
      state.stdout.queue(Print(ps1)).unwrap();
  }
}
