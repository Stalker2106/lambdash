use crossterm::{cursor, style::Print, terminal, QueueableCommand};

use crate::core::{cmdoutput::CmdOutput, core::{ShellError, ShellState}};

pub fn render_columns(state: &mut ShellState, items: &[String], padding: usize, max_rows: u16) -> Result<CmdOutput, ShellError> {
  let mut output = CmdOutput::new();
  let (cols, _) = state.termsize;

  // Calculate max item length and columns
  let max_item_length = items.iter().map(|s| s.len()).max().unwrap_or(0);
  let column_width = max_item_length + padding;
  let num_columns = cols as usize / column_width.max(1);

  // Ensure we have at least one column
  let num_columns = num_columns.max(1);

  // Calculate the maximum number of items that can fit within the row and column constraints
  let max_items = (max_rows as usize) * num_columns;

  // Print strings in columns, up to max_items
  let mut start_row = state.ps1pos.1 + 1;
  if (start_row + max_rows) > state.termsize.1 {
    let added_rows = (start_row + max_rows) - state.termsize.1;
    output.stdout.queue(terminal::ScrollUp(added_rows)).unwrap();
    start_row -= added_rows;
    state.ps1pos.1 -= added_rows;
  }
  output.stdout.queue(cursor::MoveTo(0, start_row)).unwrap();
  for (index, item) in items.iter().take(max_items).enumerate() {
      let col_position = (index % num_columns) * column_width as usize;
      let row_position = index / num_columns + start_row as usize;

      output.stdout
          .queue(cursor::MoveTo(col_position as u16, row_position as u16)).unwrap()
          .queue(Print(item)).unwrap();

      if index % num_columns == num_columns - 1 {
          output.stdout.queue(cursor::MoveToNextLine(1)).unwrap();
      }
  }
  Ok(output)
}