use std::{env, fs};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;

use crate::core::core::ShellState;
use crate::core::error::ShellError;
use crate::rendering::autocomplete::render_options;

/// Represents the state of the autocomplete feature.
///
/// # Fields
/// - `index`: The current selected index in the autocomplete suggestions, or `None` if no selection is active.
/// - `items`: A vector of strings containing the autocomplete suggestions.
pub struct AutocompleteState {
    pub index: Option<usize>,
    pub items: Vec<String>,
}

/// Handles autocomplete logic and state management.
///
/// # Fields
/// - `state`: Optional autocomplete state, containing the current suggestions and selection index.
pub struct Autocomplete {
    state: Option<AutocompleteState>,
}

impl Autocomplete {
    /// Create a new `Autocomplete` instance with no initial state.
    ///
    /// # Returns
    /// - `Autocomplete`: A new instance with `state` set to `None`.
    pub fn new() -> Autocomplete {
        Autocomplete { state: None }
    }

    /// Perform autocomplete based on the given input expression.
    ///
    /// This function handles both command and path completions, depending on the structure of the input.
    /// If there are multiple suggestions, they are stored in the autocomplete state for navigation.
    ///
    /// # Arguments
    /// - `state`: A mutable reference to the shell state, used for rendering suggestions.
    /// - `expr`: The user input expression to complete.
    ///
    /// # Returns
    /// - `Ok(Some(String))`: A single suggestion, if one is uniquely determined.
    /// - `Ok(None)`: If no suggestions are found.
    /// - `Err(ShellError)`: If an error occurs during completion or rendering.
    pub fn complete(
        &mut self,
        state: &mut ShellState,
        expr: &str,
    ) -> Result<Option<String>, ShellError> {
        let mut res: Option<String> = None;

        if let Some(astate) = self.state.as_mut() {
            if let Some(index) = astate.index.as_mut() {
                if *index < astate.items.len() - 1 {
                    *index += 1;
                } else {
                    *index = 0;
                }
                res = astate.items.get(*index).cloned();
            } else {
                astate.index = Some(0);
                res = astate.items.get(0).cloned();
            }
        } else {
            let mut results: Vec<String> = Vec::new();
            if expr.starts_with("cd ") || expr.contains('/') || expr.contains('.') {
                results = path_completion(expr);
            } else {
                results = command_completion(expr);
            }

            match results.len() {
                0 => return Ok(None),
                1 => return Ok(results.last().cloned()),
                _ => {
                    let astate = AutocompleteState {
                        index: None,
                        items: results,
                    };
                    self.state = Some(astate);
                }
            }
        }

        if let Some(astate) = &self.state {
            print_options(state, &astate).unwrap();
        }
        Ok(res)
    }

    /// Reset the autocomplete state, clearing suggestions and selections.
    ///
    /// # Arguments
    /// - `state`: A mutable reference to the shell state, used to clear the rendered suggestions.
    pub fn reset(&mut self, state: &mut ShellState) {
        self.state = None;
        state.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
    }
}

/// Perform command completion by searching the `PATH` environment variable for matching executables.
///
/// # Arguments
/// - `expr`: The input expression to match against available commands.
///
/// # Returns
/// - `Vec<String>`: A sorted vector of matching command names.
fn command_completion(expr: &str) -> Vec<String> {
    let mut available: Vec<String> = Vec::new();
    let mut searchpaths: Vec<String> = Vec::new();

    if let Ok(path) = env::var("PATH") {
        searchpaths = path.split(':').map(String::from).collect();
    }

    for searchpath in searchpaths {
        if let Ok(entries) = fs::read_dir(searchpath) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(path_str) = path.file_name().and_then(|f| f.to_str()) {
                            if path_str.starts_with(expr) {
                                available.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    available.sort();
    available
}

/// Render autocomplete suggestions to the terminal.
///
/// # Arguments
/// - `state`: A mutable reference to the shell state, used for rendering.
/// - `astate`: The current autocomplete state containing suggestions.
///
/// # Returns
/// - `Ok(())`: If the rendering succeeds.
/// - `Err(ShellError)`: If rendering fails.
fn print_options(
    state: &mut ShellState,
    astate: &AutocompleteState,
) -> Result<(), ShellError> {
    match render_options(state, astate, state.termsize.1 / 2) {
        Ok(out) => {
            if let Ok(output) = String::from_utf8(out.stdout) {
                state.stdout.queue(Print(output)).unwrap();
                Ok(())
            } else {
                Err(ShellError::ExitRequest)
            }
        }
        Err(error) => Err(error),
    }
}

/// Perform path completion by listing directory entries that match the input.
///
/// # Arguments
/// - `expr`: The input expression to match against directory entries.
///
/// # Returns
/// - `Vec<String>`: A sorted vector of matching paths.
fn path_completion(expr: &str) -> Vec<String> {
    use std::path::Path;

    let mut available = Vec::new();
    let path = Path::new(expr);
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(Path::new("."))
    };
    let prefix = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(prefix) {
                        available.push(name.to_string());
                    }
                }
            }
        }
    }

    available.sort();
    available
}
