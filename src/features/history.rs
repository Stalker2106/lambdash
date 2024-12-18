use std::{env, fs};
use std::path::{Path, PathBuf};

pub struct History {
    values: Vec<String>,
}

fn get_store_path() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        let configdir = Path::new(&home).join(".lambdash").join("history");
        return Some(configdir.to_path_buf());
    }
    return None
}

impl History {

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

    pub fn get_values(&self) -> &Vec<String> {
        return &self.values;
    }

    pub fn load() -> History {
        if let Some(config_path) = get_store_path() {
            if let Ok(data) = fs::read_to_string(config_path) {
                return History{
                    values: data.lines().map(String::from).collect()
                }
            }
        }
        return History{
            values: Vec::new()
        }
    }

    pub fn persist(&self) {
        if let Some(config_path) = get_store_path() {
            let data = self.values.join("\n");
            fs::write(&config_path, data);
        }
    }
}