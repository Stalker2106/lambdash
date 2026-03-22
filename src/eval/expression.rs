use std::vec::Vec;
use core::slice::Iter;
use std::iter::Peekable;

use crate::{core::error::StatusEnum, parser::tokenizer::{RedirectionType, Token}};

#[derive(Debug, Copy, Clone)]
pub enum ParserError {
    InvalidBackground = 228,
    InvalidPipe = 229,
    InvalidRedirection = 230
}

impl StatusEnum for ParserError {
    fn status(&self) -> u16 {
        *self as u16
    }
}

pub struct Redirection {
    pub rtype: RedirectionType,
    pub target: String
}

pub enum ExpressionGroupType {
    Single,
    Pipeline,
    Or,
    And
}

pub struct Expression {
    pub words: Vec<String>,
    pub inputs: Vec<Redirection>,
    pub outputs: Vec<Redirection>,
    pub background: bool
}

pub struct ExpressionGroup {
    pub expressions: Vec<Expression>,
    pub gtype: ExpressionGroupType
}


pub fn parse_command(tokens_iter: &mut Peekable<Iter<Token>>) -> Result<ExpressionGroup, ParserError>  {
    let mut group: ExpressionGroup = ExpressionGroup{
        expressions: Vec::new(),
        gtype: ExpressionGroupType::Single
    };
    while let Some(token) = tokens_iter.next() {
        match token {
            Token::Word(word) => {
                if let Some(cmd) = group.expressions.last_mut() {
                    cmd.words.push(word.clone());
                } else {
                    group.expressions.push(Expression{
                        words: vec![word.clone()],
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                        background: false
                    })
                }
            },
            Token::Pipe => {
                if let Some(_) = group.expressions.last() {
                    if let Some(next_token) = tokens_iter.next() {
                        match next_token {
                            Token::Word(word) => {
                                // Insert new command
                                group.expressions.push(Expression{
                                    words: vec![word.clone()],
                                    inputs: Vec::new(),
                                    outputs: Vec::new(),
                                    background: false
                                })
                            },
                            _ => return Err(ParserError::InvalidPipe)
                        }
                    } else {
                        return Err(ParserError::InvalidPipe);
                    }
                } else {
                    return Err(ParserError::InvalidPipe);
                }
            },
            Token::Redirection(rtype) => {
                if let Some(cmd) = group.expressions.last_mut() {
                    if let Some(next_token) = tokens_iter.next() {
                        match next_token {
                            Token::Word(word) => {
                                // Set current command redirection
                                let redirection = Redirection{
                                    rtype: rtype.clone(),
                                    target: (word).clone()
                                };
                                match rtype {
                                    RedirectionType::Output | RedirectionType::Append => {
                                        cmd.outputs.push(redirection);
                                    },
                                    RedirectionType::Input | RedirectionType::Heredoc => {
                                        cmd.inputs.push(redirection);
                                    }
                                }
                            },
                            _ => return Err(ParserError::InvalidPipe)
                        }
                    } else {
                        return Err(ParserError::InvalidRedirection);
                    }
                } else {
                    return Err(ParserError::InvalidRedirection);
                }
            },
            Token::Background => {
                if let Some(cmd) = group.expressions.last_mut() {
                    cmd.background = true;
                    break;
                } else {
                    return Err(ParserError::InvalidBackground)
                }
            },
            Token::CommandSeparator => {
                break;
            },
            Token::Operator(_op) => {
                unimplemented!()
            }
            _ => {
                println!("undefined token encountered");
            }
        }
    }
    return Ok(group);
}

pub fn parse_tokens(tokens: &Vec<Token>) -> Result<Vec<ExpressionGroup>, ParserError> {
    let mut parsed_groups = Vec::new();
    let mut tokens_iter = tokens.iter().peekable();

    while let Some(_) = tokens_iter.peek() {
        match parse_command(&mut tokens_iter) {
            Ok(group) => {
                match group.expressions.len() {
                    0 => (),
                    _ => parsed_groups.push(group)
                }
            },
            Err(error) => return Err(error)
        }
    }
    return Ok(parsed_groups);
}