use std::env;
use std::fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct ShellConfig {
    pub prompt: PromptConfig,
}

#[derive(Deserialize)]
pub struct PromptConfig {
    pub ps1: String,
}

pub fn load() -> ShellConfig {
    if let Some(config_path) = get_path() {
        if let Ok(data) = fs::read_to_string(config_path) {
            return toml::from_str(&data).unwrap();
        } else {
            return default();
        }
    } else {
        return default();
    }
}

fn get_path() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        let configdir = Path::new(&home).join(".lambdash").join("Config.toml");
        return Some(configdir.to_path_buf());
    }
    return None
}

fn default() -> ShellConfig {
    return ShellConfig{
        prompt: PromptConfig{
            ps1: "[color=yellow]λsh[/color] $PWD [color=red]($?)[/color] >".to_string(),
        }
    };
}

impl ShellConfig {

}