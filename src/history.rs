
pub struct History {
    values: Vec<String>,
}

impl History {
    pub fn new() -> History {
        return History{
            values: Vec::new()
        }
    }

    pub fn get(&self, index: usize) -> &str {
        if let Some(value) = self.values.get(index) {
            return value;
        }
        return "";
    }

    pub fn submit(&mut self, value: &str) {
        if let Some(index) = self.values.iter().position(|v| v == value) {
            self.values.remove(index);
        }
        self.values.push(value.to_string());
    }

    pub fn get_first_index(&self) -> Option<usize> {
        if self.values.len() <= 0 {
            return None;
        }
        return Some(self.values.len() - 1);
    }
}