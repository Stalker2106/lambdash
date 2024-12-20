use std::{env, fs};
use std::path::{Path, PathBuf};

/// A structure representing a history of submitted commands or inputs.
///
/// # Fields
/// - `values`: A vector of strings holding the history entries, in order.
pub struct History {
    values: Vec<String>,
}

/// Get the file path where the history should be stored.
///
/// This function checks the `HOME` environment variable and constructs a path
/// to `.lambdash/history` within the user's home directory. If the `HOME`
/// variable is not set, the function returns `None`.
///
/// # Returns
/// - `Some(PathBuf)`: The resolved path to the history file, if `HOME` is set.
/// - `None`: If the `HOME` environment variable is not set.
fn get_store_path() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        let configdir = Path::new(&home).join(".lambdash").join("history");
        return Some(configdir.to_path_buf());
    }
    return None;
}

impl History {
    /// Retrieve a specific entry from the history by its index.
    ///
    /// # Arguments
    /// - `index`: The index of the history entry to retrieve.
    ///
    /// # Returns
    /// - `&str`: The history entry at the given index, or an empty string if the index is out of bounds.
    pub fn get(&self, index: usize) -> &str {
        if let Some(value) = self.values.get(index) {
            return value;
        }
        return "";
    }

    /// Add a new entry to the history.
    ///
    /// If the entry already exists in the history, it is removed first to
    /// prevent duplicates. The new entry is then appended to the end of the history.
    ///
    /// # Arguments
    /// - `value`: The entry to add to the history.
    pub fn submit(&mut self, value: &str) {
        if let Some(index) = self.values.iter().position(|v| v == value) {
            self.values.remove(index);
        }
        self.values.push(value.to_string());
    }

    /// Get the index of the first (most recent) entry in the history.
    ///
    /// # Returns
    /// - `Some(usize)`: The index of the first entry, if the history is not empty.
    /// - `None`: If the history is empty.
    pub fn get_first_index(&self) -> Option<usize> {
        if self.values.len() <= 0 {
            return None;
        }
        return Some(self.values.len() - 1);
    }

    /// Get a reference to the entire history.
    ///
    /// # Returns
    /// - `&Vec<String>`: A reference to the vector of history entries.
    pub fn get_values(&self) -> &Vec<String> {
        return &self.values;
    }

    /// Load the history from the storage file.
    ///
    /// This function attempts to read the history from the file at the path
    /// specified by `get_store_path()`. If the file does not exist or cannot
    /// be read, an empty history is returned.
    ///
    /// # Returns
    /// - `History`: A new `History` instance populated with entries from the file,
    ///   or an empty history if the file could not be read.
    pub fn load() -> History {
        if let Some(config_path) = get_store_path() {
            if let Ok(data) = fs::read_to_string(config_path) {
                return History {
                    values: data.lines().map(String::from).collect(),
                };
            }
        }
        return History {
            values: Vec::new(),
        };
    }

    /// Persist the history to the storage file.
    ///
    /// This function writes the current history to the file specified by
    /// `get_store_path()`. If the parent directory of the file does not exist,
    /// it is created. The history is written as newline-separated entries.
    pub fn persist(&self) {
        if let Some(config_path) = get_store_path() {
            // Create the config dir if missing
            if let Some(dir) = config_path.parent() {
                if !dir.exists() {
                    fs::create_dir_all(dir).unwrap();
                }
            }
            // Write history
            let data = self.values.join("\n");
            fs::write(&config_path, data).unwrap();
        }
    }
}
