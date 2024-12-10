extern crate console;

mod shell;
mod tokenizer;
mod executor;
mod builtins;

use shell::Shell;
use tokenizer::tokenize;
use executor::execute_tokens;
use executor::preprocess_tokens;

fn main() {
    let running = true;
    let mut shell = Shell::new();
    while running {
        shell.print_prompt();
        shell.poll_input();
        shell.historize_input();
        match tokenize(&shell.input) {
            Ok(mut tokens) => {
                if tokens.len() > 0 {
                    preprocess_tokens(&mut tokens);
                    match execute_tokens(&mut shell, &tokens) {
                        Ok(s) => shell.status = s,
                        Err(error) => {
                            shell.status = error.status;
                            println!("Error: {}", error.details);
                        }
                    }
                }
            },
            Err(error) => println!("Error: {}", error.details)
        };
        shell.input.clear();
    }
}