use std::io::{Read, Write};
use std::fs::File;
use std::fs::OpenOptions;

pub enum FSError {
    IOError
}

// Out

fn output_to_file_truncate(output: &Vec<u8>, path: &str) -> Result<(), FSError> {
    match File::create(path) {
        Ok(mut file) => {
            file.write_all(output);
            return Ok(())
        },
        Err(_) => Err(FSError::IOError)
    }
}

fn output_to_file(output: &Vec<u8>, path: &str) -> Result<(), FSError> {
    match OpenOptions::new().write(true).create(true).append(true).open(path) {
        Ok(mut file) => {
            file.write_all(output);
            return Ok(())
        },
        Err(_) => Err(FSError::IOError)
    }
}

// In

fn file_as_input(path: &str) -> Result<Vec<u8>, FSError> {
    match File::open(path) {
        Ok(mut file) => {
            let mut buffer: Vec<u8> = Vec::new();
            match file.read_to_end(&mut buffer) {
                Ok(_) => return Ok(buffer),
                Err(error) => Err(FSError::IOError)
            }
        },
        Err(_) => Err(FSError::IOError)
    }
}