use std::io::{Read, Write};
use std::fs::File;
use std::fs::OpenOptions;

pub enum FSError {
    IOError
}

pub fn open_file(path: &str, truncate: bool) -> Result<File, FSError> {
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    if truncate {
        options.truncate(true);
    } else {
        options.append(true);
    }
    match options.open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(FSError::IOError),
    }
}

// Out

pub fn write_output_to_file(output: &Vec<u8>, path: &str, truncate: bool) -> Result<(), FSError> {
    match open_file(path, truncate) {
        Ok(mut file) => {
            file.write_all(output).unwrap();
            return Ok(())
        },
        Err(error) => Err(error)
    }
}

// In

pub fn read_file_as_input(path: &str) -> Result<Vec<u8>, FSError> {
    match File::open(path) {
        Ok(mut file) => {
            let mut buffer: Vec<u8> = Vec::new();
            match file.read_to_end(&mut buffer) {
                Ok(_) => return Ok(buffer),
                Err(_) => Err(FSError::IOError)
            }
        },
        Err(_) => Err(FSError::IOError)
    }
}