use crossterm::cursor;

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
        if let Some(stash) = &self.input_stash {
            self.set_input(&stash.clone());
        }
    }

    pub fn clear_stash(&mut self) {
        if let Some(stash) = &mut self.input_stash {
            stash.clear();
        }
    }

    // cursor

    pub fn get_cursor(&self) -> usize {
        return self.cursor;
    }
    pub fn get_cursor_offset(&self) -> (usize, usize) {
        let sub_input = &self.input[..self.cursor];
        let newline_count = sub_input.matches('\n').count();
        if let Some(pos) = sub_input.rfind('\n') {
            return (self.cursor - pos - 1, newline_count)
        } else {
            return (self.cursor, 0)
        }
    }

    pub fn move_cursor(&mut self, pos: usize) -> bool {
        if pos <= self.input.len() {
            self.cursor = pos;
            return true;
        }
        return false;
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