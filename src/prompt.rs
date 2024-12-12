use crate::{core::ShellState, promptscript::print_expr};

pub struct Prompt {
    input_stash: Option<String>,
    input: String,
    cursor: usize,
}

impl Prompt {
    pub fn new() -> Prompt {
        return Prompt{
            input_stash: None,
            input: String::new(),
            cursor: 0
        }
    }

    // ps1

    pub fn print_ps1(&self, state: &mut ShellState) {
        let ps1 = state.config.prompt.ps1.clone();
        print_expr(state, &ps1);
    }

    // input

    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn append_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn remove_char(&mut self) -> bool {
        if self.cursor <= 0 {
            return false;
        }
        self.cursor -= 1;
        self.input.remove(self.cursor);
        return true;
    }

    pub fn set_input(&mut self, str: &str) {
        self.input = str.to_string();
        self.cursor = str.len();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    pub fn has_input(&self) -> bool {
        return !self.input.is_empty();
    }

    pub fn get_input(&self) -> &String {
        return &self.input;
    }

    // stash

    pub fn stash_input(&mut self) {
        self.input_stash = Some(self.input.clone());
    }

    pub fn unstash_input(&mut self) {
        if self.input_stash.is_some() {
            self.set_input(&self.input_stash.clone().unwrap());
        }
    }

    // cursor

    pub fn get_cursor(&self) -> usize {
        return self.cursor;
    }

    pub fn move_cursor_left(&mut self, amount: usize) -> Option<usize> {
        if self.cursor > 0 {
            let newpos: i32 = (self.cursor - amount) as i32;
            if newpos < 0 {
                self.cursor = 0;
                return Some(amount - newpos as usize);
            }
            self.cursor = newpos as usize;
            return Some(amount);
        }
        return None;
    }

    pub fn move_cursor_right(&mut self, amount: usize) -> Option<usize> {
        if self.cursor < self.get_input().len() {
            let newpos = self.cursor + amount;
            if newpos > self.get_input().len() {
                let diff = newpos - self.cursor;
                self.cursor = self.get_input().len();
                return Some(diff);
            }
            self.cursor = newpos;
            return Some(amount);
        }
        return None;
    }

}