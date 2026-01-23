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
  let (termcols, termrows) = terminal::size().unwrap();
  state.stdout.queue(cursor::MoveTo(*ps1col, *ps1row)).unwrap();
  let input_lines: Vec<String> = get_prompt_lines(input);
  if termrows - *ps1row < input_lines.len() as u16 {
    *ps1row = *ps1row - (input_lines.len() - 1) as u16;
  }
  for input_line in input_lines {
    state.stdout.queue(Print(input_line)).unwrap();
  }
}

pub fn get_prompt_lines(input: &String) -> Vec<String> {
  let (termcols, termrows) = terminal::size().unwrap();
  let input_lines: Vec<&str> = input.split('\n').collect();
  let mut lines = Vec::new();
  for (row, input_line) in input_lines.iter().enumerate() {
      if row < input_lines.len() - 1 {
          lines.push(input_line.to_string());
      } else {
        let mut offset = 0;
        while offset < input_line.len() {
            let chunk_end = std::cmp::min(offset + termcols as usize, input_line.len());
            lines.push(input_line[offset..chunk_end].to_string());
            offset = chunk_end;
        }
      }
  }
  return lines;
}

pub fn align_cursor_with_prompt(state: &mut ShellState, prompt: &Prompt) {
  let (ps1col, ps1row) = state.ps1pos;
  let (curcol, currow) = prompt.get_cursor_offset(state.ps1pos);
  state.stdout.queue(cursor::MoveTo(curcol as u16, ps1row + currow as u16)).unwrap();
}

pub fn print_prompt(state: &mut ShellState, prompt: &Prompt) {
  let ps1out = eval_ps(state, &prompt.ps1);
  if let Ok(ps1) = String::from_utf8(ps1out.stdout) {
      state.stdout.queue(Print(ps1)).unwrap();
  }
}