use crossterm::{cursor, style::{Print, ResetColor, SetBackgroundColor}, terminal, QueueableCommand};

use crate::core::{cmdoutput::CmdOutput, core::{ShellError, ShellState}};
use crate::features::autocomplete::AutocompleteState;

pub fn render_options(state: &mut ShellState, astate: &AutocompleteState, max_rows: u16) -> Result<CmdOutput, ShellError> {
  let mut output = CmdOutput::new();
  let (cols, _) = state.termsize;
  // Calculate max item length and columns
  let max_item_length = astate.items.iter().map(|s| s.len()).max().unwrap_or(0);
  let column_width = max_item_length;
  let num_columns = cols as usize / column_width.max(1);
  // Calculate the number of rows required to display all items
  let num_rows_needed = (astate.items.len() + num_columns - 1) / num_columns; // Ceil division
  let rows_to_display = num_rows_needed.min(max_rows as usize);
  let mut start_row = state.ps1pos.1 + 1;
  // Adjust starting position if printing would overflow the screen
  if (start_row + rows_to_display as u16) > state.termsize.1 {
      let added_rows = (start_row + rows_to_display as u16) - state.termsize.1;
      output.stdout.queue(terminal::ScrollUp(added_rows)).unwrap();
      start_row -= added_rows;
      state.ps1pos.1 -= added_rows;
  }
  let mut selected_index: i32 = -1;
  if let Some(sidx) = astate.index {
    selected_index = sidx as i32;
  }
  // Print
  output.stdout.queue(cursor::MoveTo(0, start_row)).unwrap();
  for (index, item) in astate.items.iter().enumerate() {
      let col_position = (index % num_columns) * column_width;
      let row_position = index / num_columns + start_row as usize;
      // Stop if exceeding rows to display
      if row_position >= (start_row as usize + rows_to_display) {
          break;
      }
      output.stdout.queue(cursor::MoveTo(col_position as u16, row_position as u16)).unwrap();
      render_option(&mut output, item, index as i32 == selected_index);
      if index % num_columns == num_columns - 1 {
          output.stdout.queue(cursor::MoveToNextLine(1)).unwrap();
      }
  }
  Ok(output)
}

fn render_option(output: &mut CmdOutput, option: &String, selected: bool) {
  if selected {
    output.stdout.queue(SetBackgroundColor(crossterm::style::Color::White)).unwrap();
  }
  output.stdout.queue(Print(option)).unwrap();
  if selected {
    output.stdout.queue(ResetColor).unwrap();
  }
}