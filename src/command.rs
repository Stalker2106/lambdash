use std::vec::Vec;
use core::slice::Iter;
use std::iter::Peekable;

use crate::{tokenizer::{RedirectionType, Token}};

#[derive(Debug)]
pub enum ParseError {
    InvalidBackground,
    InvalidPipe,
    InvalidRedirection
}

pub struct Redirection {
    pub rtype: RedirectionType,
    pub target: String
}


pub struct Command {
    pub words: Vec<String>,
    pub redirections: Vec<Redirection>,
    pub background: bool
}


pub fn parse_command(tokens_iter: &mut Peekable<Iter<Token>>) -> Result<Vec<Command>, ParseError>  {
    let mut commands: Vec<Command> = Vec::new();
    while let Some(token) = tokens_iter.next() {
        match token {
            Token::Word(word) => {
                if let Some(cmd) = commands.last_mut() {
                    cmd.words.push(word.clone());
                } else {
                    commands.push(Command{
                        words: vec![word.clone()],
                        redirections: Vec::new(),
                        background: false
                    })
                }
            },
            Token::Pipe => {
                if let Some(_) = commands.last() {
                    if let Some(next_token) = tokens_iter.next() {
                        match next_token {
                            Token::Word(word) => {
                                // Insert new command
                                commands.push(Command{
                                    words: vec![word.clone()],
                                    redirections: Vec::new(),
                                    background: false
                                })
                            },
                            _ => return Err(ParseError::InvalidPipe)
                        }
                    } else {
                        return Err(ParseError::InvalidPipe);
                    }
                } else {
                    return Err(ParseError::InvalidPipe);
                }
            },
            Token::Redirection(rtype) => {
                if let Some(cmd) = commands.last_mut() {
                    if let Some(next_token) = tokens_iter.next() {
                        match next_token {
                            Token::Word(word) => {
                                // Set current command redirection
                                cmd.redirections.push(Redirection{
                                    rtype: rtype.clone(),
                                    target: (word).clone()
                                });
                            },
                            _ => return Err(ParseError::InvalidPipe)
                        }
                    } else {
                        return Err(ParseError::InvalidRedirection);
                    }
                } else {
                    return Err(ParseError::InvalidRedirection);
                }
            },
            Token::Background => {
                if let Some(cmd) = commands.last_mut() {
                    cmd.background = true;
                    break;
                } else {
                    return Err(ParseError::InvalidBackground)
                }
            },
            Token::CommandSeparator => {
                break;
            }
            _ => {
                println!("undefined token encountered");
            }
        }
    }
    return Ok(commands);
}

pub fn parse_tokens(tokens: &Vec<Token>) -> Result<Vec<Vec<Command>>, ParseError> {
    let mut parsed_commands = Vec::new();
    let mut tokens_iter = tokens.iter().peekable();

    while let Some(_) = tokens_iter.peek() {
        match parse_command(&mut tokens_iter) {
            Ok(cmds) => {
                match cmds.len() {
                    0 => (),
                    _ => parsed_commands.push(cmds)
                }
            },
            Err(error) => return Err(error)
        }
    }
    return Ok(parsed_commands);
}