
pub struct History {
    values: Vec<String>,
}

impl History {
    pub fn new() -> History {
        return History{
            values: Vec::new()
        }
    }

    pub fn submit_value(&mut self, value: &str) {
        if let Some(index) = self.values.iter().position(|v| v == value) {
            self.values.remove(index);
        }
        self.values.push(value.to_string());
    }
}