pub struct Prompt {
    input_stash: Option<String>,
    input: String,
    cursor: usize,
}

pub enum CursorPosition {
    Origin,
    End
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
        self.cursor += c.len_utf8();
    }

    pub fn append_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn remove_char(&mut self, back: bool) -> bool {
        if back && self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
            return true;
        } else if !back && self.cursor < self.input.len() {
            self.input.remove(self.cursor);
            return true;
        }
        return false;
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
        return self.input[..self.cursor].chars().count()
    }

    pub fn get_cursor_offset(&self) -> (usize, usize) {
        let input_until_cursor = &self.input[..self.cursor];
        let newline_count = input_until_cursor.matches('\n').count();
        let mut column_index = self.cursor;
        if let Some(pos) = input_until_cursor.rfind('\n') {
            column_index -= pos - 1;
        }
        return (self.input[..column_index].chars().count(), newline_count)
    }

    pub fn move_cursor(&mut self, pos: CursorPosition) -> bool {
        match pos {
            CursorPosition::Origin => {
                if self.cursor != 0 {
                    self.cursor = 0;
                    return true;
                }
            },
            CursorPosition::End => {
                if self.cursor != self.input.len()-1 {
                    self.cursor = self.input.len()-1;
                    return true;
                }
            }
        }
        return false;
    }

    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let mut local_cursor = self.cursor - 1;
        while local_cursor >= 0 {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor -= 1;
            } else {
                self.cursor = local_cursor;
                return true;
            }
        }
        return false;
    }

    pub fn move_cursor_right(&mut self) -> bool {
        if self.cursor == self.input.len() {
            return false;
        }
        let mut local_cursor = self.cursor + 1;
        while local_cursor <= self.input.len() {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor += 1;
            } else {
                self.cursor = local_cursor;
                return true;
            }
        }
        return false;
    }

}