use unic_emoji_char::is_emoji;

/// Represents the state and behavior of the shell prompt.
///
/// # Fields
/// - `input_stash`: Optional backup of the input string for temporary storage.
/// - `input`: The current input string being edited in the prompt.
/// - `cursor`: The current position of the cursor within the input string.
/// - `ps1`: The prompt string displayed at the beginning of the line.
pub struct Prompt {
    input_stash: Option<String>,
    input: String,
    cursor: usize,
    pub ps1: String,
}

/// Defines possible positions for the cursor.
///
/// - `Origin`: Move the cursor to the start of the input.
/// - `End`: Move the cursor to the end of the input.
pub enum CursorPosition {
    Origin,
    End,
}

/// Describes cursor movement granularity.
///
/// - `One`: Move the cursor by a single character.
/// - `Word`: Move the cursor by an entire word.
#[derive(PartialEq)]
pub enum CursorMovement {
    One,
    Word,
}

impl Prompt {
    /// Creates a new `Prompt` instance.
    ///
    /// # Arguments
    /// - `ps1script`: The string to display as the prompt (e.g., `$ `).
    ///
    /// # Returns
    /// - A new `Prompt` with an empty input and cursor at position 0.
    pub fn new(ps1script: &str) -> Prompt {
        Prompt {
            input_stash: None,
            input: String::new(),
            cursor: 0,
            ps1: ps1script.to_string(),
        }
    }

    // --- Input Management ---

    /// Adds a character at the current cursor position.
    ///
    /// # Arguments
    /// - `c`: The character to add.
    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Removes a character from the input.
    ///
    /// # Arguments
    /// - `back`: If `true`, remove the character before the cursor. If `false`, remove the character at the cursor.
    ///
    /// # Returns
    /// - `true` if a character was removed, `false` otherwise.
    pub fn remove_char(&mut self, back: bool) -> bool {
        if back && self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
            true
        } else if !back && self.cursor < self.input.len() {
            self.input.remove(self.cursor);
            true
        } else {
            false
        }
    }

    /// Replaces the current input with a new string.
    ///
    /// # Arguments
    /// - `str`: The new input string.
    pub fn set_input(&mut self, str: &str) {
        self.input = str.to_string();
        self.cursor = str.len();
    }

    /// Clears the current input and resets the cursor position.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    /// Truncates the input at the current cursor position.
    ///
    /// # Returns
    /// - `true` if the input was truncated, `false` if the cursor was already at the end.
    pub fn truncate_input(&mut self) -> bool {
        if self.cursor == self.input.len() {
            false
        } else {
            self.input.truncate(self.cursor);
            true
        }
    }

    /// Checks if the input is non-empty.
    ///
    /// # Returns
    /// - `true` if the input has content, `false` otherwise.
    pub fn has_input(&self) -> bool {
        !self.input.is_empty()
    }

    /// Retrieves the current input string.
    ///
    /// # Returns
    /// - A reference to the input string.
    pub fn get_input(&self) -> &String {
        &self.input
    }

    /// Calculates the number of rows the input spans based on newline characters.
    ///
    /// # Returns
    /// - The number of rows in the input.
    pub fn get_input_rows(&self) -> usize {
        1 + self.input.matches('\n').count()
    }

    // --- Input Stash Management ---

    /// Stashes the current input, preserving it for later use.
    pub fn stash_input(&mut self) {
        self.input_stash = Some(self.input.clone());
    }

    /// Restores the stashed input, replacing the current input.
    pub fn unstash_input(&mut self) {
        if let Some(stash) = &self.input_stash {
            self.set_input(&stash.clone());
        }
    }

    /// Clears the stashed input.
    pub fn clear_stash(&mut self) {
        if let Some(stash) = &mut self.input_stash {
            stash.clear();
        }
    }

    // --- Cursor Management ---

    /// Calculates the cursor's offset in terms of column and row.
    ///
    /// # Returns
    /// - `(column_index, row_index)` where `column_index` is the cursor's column position and `row_index` is the number of newlines before the cursor.
    pub fn get_cursor_offset(&self) -> (usize, usize) {
        let input_until_cursor = &self.input[..self.cursor];
        let newline_count = input_until_cursor.matches('\n').count();
        let mut column_index = input_until_cursor
            .chars()
            .map(|c| if is_emoji(c) { 2 } else { 1 })
            .sum::<usize>();

        if let Some(pos) = input_until_cursor.rfind('\n') {
            let input_newline_cursor = &input_until_cursor[pos + 1..self.cursor];
            column_index = input_newline_cursor
                .chars()
                .map(|c| if is_emoji(c) { 2 } else { 1 })
                .sum::<usize>();
        }

        (column_index, newline_count)
    }

    /// Moves the cursor to a specific position.
    ///
    /// # Arguments
    /// - `pos`: The target cursor position (`Origin` or `End`).
    ///
    /// # Returns
    /// - `true` if the cursor moved, `false` otherwise.
    pub fn move_cursor(&mut self, pos: CursorPosition) -> bool {
        match pos {
            CursorPosition::Origin => {
                if self.cursor != 0 {
                    self.cursor = 0;
                    true
                } else {
                    false
                }
            }
            CursorPosition::End => {
                if self.cursor != self.input.len() {
                    self.cursor = self.input.len();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Moves the cursor to the left.
    ///
    /// # Arguments
    /// - `movement`: Specifies whether to move by a single character (`One`) or by a word (`Word`).
    ///
    /// # Returns
    /// - The number of positions the cursor moved.
    pub fn move_cursor_left(&mut self, movement: CursorMovement) -> usize {
        if self.cursor == 0 {
            return 0;
        }

        let mut local_cursor = self.cursor - 1;
        loop {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor -= 1;
            } else {
                if movement == CursorMovement::Word
                    && local_cursor > 0
                    && self.input[local_cursor..]
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .is_alphanumeric()
                {
                    local_cursor -= 1;
                    continue;
                }
                let diff = self.cursor - local_cursor;
                self.cursor = local_cursor;
                return diff;
            }
        }
    }

    /// Moves the cursor to the right.
    ///
    /// # Arguments
    /// - `movement`: Specifies whether to move by a single character (`One`) or by a word (`Word`).
    ///
    /// # Returns
    /// - The number of positions the cursor moved.
    pub fn move_cursor_right(&mut self, movement: CursorMovement) -> usize {
        if self.cursor == self.input.len() {
            return 0;
        }

        let mut local_cursor = self.cursor + 1;
        loop {
            if !self.input.is_char_boundary(local_cursor) {
                local_cursor += 1;
            } else {
                if movement == CursorMovement::Word
                    && local_cursor < self.input.len()
                    && self.input[local_cursor..]
                        .chars()
                        .next()
                        .unwrap_or_default()
                        .is_alphanumeric()
                {
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
