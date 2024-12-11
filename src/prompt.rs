pub struct Prompt {
    ps1: String,
    input_stash: Option<String>,
    input: String,
    cursor: usize,
}

impl Prompt {
    pub fn new() -> Prompt {
        return Prompt{
            input_stash: None,
            input: String::new(),
            ps1: "λsh $CWD ($?) >".to_string(),
            cursor: 0
        }
    }

    // ps1

    pub fn get_ps1(&self) -> &String {
        return &self.ps1;
    }

    // input

    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn remove_char(&mut self) {
        if self.cursor <= 0 {
            return;
        }
        self.input.remove(self.cursor);
        self.cursor -= 1;
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
            self.input = self.input_stash.clone().unwrap();
        }
    }

    // cursor

    pub fn get_cursor(&self) -> usize {
        return self.cursor;
    }

    pub fn move_cursor(&mut self, amount: i16) {
        if amount < 0 && self.cursor >= amount.abs() as usize {
            self.cursor = (self.cursor as i16 + amount) as usize;
        } else if amount >= 0 {
            self.cursor = (self.cursor as i16 + amount) as usize;
        }
    }

}