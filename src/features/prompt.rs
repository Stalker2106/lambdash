use unic_emoji_char::is_emoji;

pub struct Prompt {
    input_stash: Option<String>,
    input: String,
    cursor: usize,
    pub ps1: String,
}

pub enum CursorPosition {
    Origin,
    End
}

#[derive(PartialEq)]
pub enum CursorMovement {
    One,
    Word
}

impl Prompt {
    pub fn new(ps1script: &str) -> Prompt {
        return Prompt{
            input_stash: None,
            input: String::new(),
            cursor: 0,
            ps1: ps1script.to_string()
        }
    }

    // input

    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
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

    pub fn truncate_input(&mut self) -> bool {
        if self.cursor == self.input.len() {
            return false;
        }
        self.input.truncate(self.cursor);
        return true;
    }

    pub fn has_input(&self) -> bool {
        return !self.input.is_empty();
    }

    pub fn get_input(&self) -> &String {
        return &self.input;
    }

    pub fn get_input_rows(&self) -> usize {
        return 1 + self.input.matches('\n').count();
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
    pub fn get_cursor_offset(&self) -> (usize, usize) {
        let input_until_cursor = &self.input[..self.cursor];
        let newline_count = input_until_cursor.matches('\n').count();
        let mut column_index = input_until_cursor.chars().map(|c| if is_emoji(c) { 2 } else { 1 }).sum::<usize>();
        if let Some(pos) = input_until_cursor.rfind('\n') {
            let input_newline_cursor = &input_until_cursor[pos + 1..self.cursor];
            column_index = input_newline_cursor.chars().map(|c| if is_emoji(c) { 2 } else { 1 }).sum::<usize>();
        }
        return (column_index, newline_count)
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

    pub fn move_cursor_left(&mut self, movement: CursorMovement) -> usize {
        if self.cursor == 0 {
            return 0;
        }
        let mut local_cursor = self.cursor - 1;
        loop {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor -= 1;
            } else {
                if movement == CursorMovement::Word && local_cursor > 0 && self.input[local_cursor..].chars().next().unwrap_or_default().is_alphanumeric() {
                    local_cursor -= 1;
                    continue;
                }
                let diff = self.cursor - local_cursor;
                self.cursor = local_cursor;
                return diff;
            }
        }
    }

    pub fn move_cursor_right(&mut self, movement: CursorMovement) -> usize {
        if self.cursor == self.input.len() {
            return 0;
        }
        let mut local_cursor = self.cursor + 1;
        loop {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor += 1;
            } else {
                if movement == CursorMovement::Word && local_cursor < self.input.len() && self.input[local_cursor..].chars().next().unwrap_or_default().is_alphanumeric() {
                    local_cursor += 1;
                    continue;
                }
                let diff = local_cursor - self.cursor;
                self.cursor = local_cursor;
                return diff;
            }
        }
    }

}