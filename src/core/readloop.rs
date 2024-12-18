use crossterm::{cursor, event::{read, Event, KeyCode, KeyEvent, KeyModifiers}, terminal::{Clear, ClearType}, QueueableCommand};

use crate::{features::{autocomplete::Autocomplete, prompt::{CursorMovement, CursorPosition, Prompt}}, rendering::prompt::{align_cursor_with_prompt, clear_prompt_input, print_prompt_input}};

use super::core::ShellState;

pub fn handle_ctrl_modifiers(state: &mut ShellState, autocomplete: &mut Autocomplete, prompt: &mut Prompt, event: KeyEvent) -> (i32, bool) {
  match event.code {
    KeyCode::Char(c) => {
        match c {
            'c' => {
                prompt.clear_stash();
                prompt.clear_input();
                autocomplete.reset(state);
                return (0, true);
            },
            'd' => {
                return (0, !prompt.has_input());
            }
            'l' => {
                state.stderr.queue(Clear(ClearType::All)).unwrap()
                            .queue(cursor::MoveTo(0,0)).unwrap();
                state.stdout.queue(cursor::MoveTo(0,0)).unwrap()
                            .queue(Clear(ClearType::All)).unwrap();
                prompt.stash_input();
                prompt.clear_input();
                return (0, true);
            },
            'k' => {
                if prompt.truncate_input() {
                    state.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
                }
                return (0, false);
            }
            _ => return (0, false)
        }
    },
    _ => return (0, false)
  }
}

pub fn handle_alt_modifiers(state: &mut ShellState, prompt: &mut Prompt, event: KeyEvent) -> (i32, bool) {
  match event.code {
    KeyCode::Left => {
        let diff = prompt.move_cursor_left(CursorMovement::Word);
        if diff > 0 {
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    KeyCode::Right => {
        let diff = prompt.move_cursor_right(CursorMovement::Word);
        if diff > 0 {
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    _ => return (0, false)
}
}

pub fn handle_input(state: &mut ShellState, autocomplete: &mut Autocomplete, prompt: &mut Prompt, history_idx: &mut Option<usize>, event: KeyEvent) -> (i32, bool) {
  match event.code {
    KeyCode::Char(c) => {
        prompt.add_char(c);
        autocomplete.reset(state);
        clear_prompt_input(state, &prompt);
        print_prompt_input(state, prompt.get_input());
        align_cursor_with_prompt(state, prompt);
        return (1, false);
    }
    KeyCode::Home => {
        if prompt.move_cursor(CursorPosition::Origin) {
            let (ps1col, ps1row) = state.ps1pos;
            state.stdout.queue(cursor::MoveTo(ps1col, ps1row)).unwrap();
        }
        return (0, false);
    },
    KeyCode::End => {
        if prompt.move_cursor(CursorPosition::End) {
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    KeyCode::Left => {
        let diff = prompt.move_cursor_left(CursorMovement::One);
        if diff > 0 {
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    KeyCode::Right => {
        let diff = prompt.move_cursor_right(CursorMovement::One);
        if diff > 0 {
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
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
        return (0, false);
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
        return (0, false);
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
        return (0, false);
    },
    KeyCode::Delete => {
        if prompt.remove_char(false) {
            clear_prompt_input(state, &prompt);
            print_prompt_input(state, prompt.get_input());
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    KeyCode::Backspace => {
        if prompt.remove_char(true) {
            autocomplete.reset(state);
            clear_prompt_input(state, &prompt);
            print_prompt_input(state, prompt.get_input());
            align_cursor_with_prompt(state, prompt);
        }
        return (0, false);
    },
    KeyCode::Enter => {
        autocomplete.reset(state);
        return (1, true);
    },
    _ => return (0, false)
  }
}

pub fn handle_event(state: &mut ShellState, autocomplete: &mut Autocomplete, prompt: &mut Prompt, history_idx: &mut Option<usize>, event: Event) -> (i32, bool) {
  match event {
    Event::Resize(width, height) => {
      state.update_size(width, height);
      return (0, false);
    },
    Event::Key(event) => {
        if event.modifiers.contains(KeyModifiers::CONTROL) {
          return handle_ctrl_modifiers(state, autocomplete, prompt, event);
        } else if event.modifiers.contains(KeyModifiers::ALT) {
          return handle_alt_modifiers(state, prompt, event);
        } else {
          return handle_input(state, autocomplete, prompt, history_idx, event);
        }
    }
    _ => return (0, false)
  }
}

pub fn prompt_readloop(state: &mut ShellState, autocomplete: &mut Autocomplete, prompt: &mut Prompt, history_idx: &mut Option<usize>) -> i32 {
  let mut chars_read = -1;
  crossterm::terminal::enable_raw_mode().unwrap();
  loop {
      if let Ok(event) = read() {
        let (chars, finished) = handle_event(state, autocomplete, prompt, history_idx, event);
        chars_read += chars;
        if finished { break; }
      }
      state.stdout.flush().unwrap();
  }
  crossterm::terminal::disable_raw_mode().unwrap();
  return chars_read;
}
