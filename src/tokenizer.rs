use std::iter::Peekable;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

pub enum Token {
    Word(String),
    Pipe,         // |
    Background,   // &
    Subcommand(Vec<Token>), //() or ``
    Redirection(String), // >, <, >>, <<
    Variable(String), // $VAR or ${VAR}
    Operator(String), // && or ||
    CommandSeparator,   //;
}

const RESERVED_CHARS: &str = "\"';|&$<>";

enum TokenizationErrorType {
    InvalidSyntax = 123,
    UnmatchedCharacter = 127
}

#[derive(Debug)]
pub struct TokenizationError {
    pub details: String,
    pub status: ExitStatus,
}

impl TokenizationError {
    fn new(code: TokenizationErrorType, msg: String) -> TokenizationError {
        TokenizationError{
            status: ExitStatus::from_raw(code as i32),
            details: msg.to_string()
        }
    }
}

pub fn parse_variable(iter: &mut Peekable<std::str::Chars>, index: &mut i32) -> String {
    let mut var = String::new();
    while let Some(&next) = iter.peek() {
        if next.is_alphanumeric() || next == '_' || (var.len() == 0 && next == '?') {
            var.push(iter.next().unwrap());
            *index += 1;
        } else {
            break;
        }
    }
    return var;
}

fn parse_until_next(iter: &mut Peekable<std::str::Chars>, index: &mut i32, closing_char: char) -> Result<String, TokenizationErrorType> {
    let mut closed = false;
    let mut content = String::new();
    while let Some(&next) = iter.peek() {
        if next == closing_char {
            closed = true;
            iter.next();
            *index += 1;
            break;
        }
        content.push(iter.next().unwrap());
    }
    if !closed {
        return Err(TokenizationErrorType::UnmatchedCharacter);
    }
    return Ok(content);
}

pub fn tokenize(expr: &String) -> Result<Vec<Token>, TokenizationError> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    let mut index = 0;
    while let Some(c) = chars.next() {
        index += 1;
        match c {
            '|' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Operator("||".to_string()));
                } else {
                    tokens.push(Token::Pipe);
                }
            },
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Operator("&&".to_string()));
                } else {
                    tokens.push(Token::Background);
                }
            },
            '>' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection(">>".to_string()));
                } else {
                    tokens.push(Token::Redirection(">".to_string()));
                }
            },
            '<' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection("<<".to_string()));
                } else {
                    tokens.push(Token::Redirection("<".to_string()));
                }
            },
            '$' => {
                tokens.push(Token::Variable(parse_variable(&mut chars, &mut index)));
            },
            '\'' | '"' => match parse_until_next(&mut chars, &mut index, c) {
                Ok(content) => tokens.push(Token::Word(content)),
                Err(error_type) => return Err(TokenizationError::new(error_type, format!("unterminated quote {} opened at position {}", c, index)))
            },
            '`' | '(' => match parse_until_next(&mut chars, &mut index, if c == '(' { ')' } else { c }) {
                Ok(content) => {
                    match tokenize(&content) {
                        Ok(subtokens) => tokens.push(Token::Subcommand(subtokens)),
                        Err(error) => return Err(error)
                    }
                },
                Err(error_type) => return Err(TokenizationError::new(error_type, format!("unmatched character {} at position {}", c, index)))
            },
            ';' => tokens.push(Token::CommandSeparator),
            c if c.is_whitespace() => continue, 
            _ => {
                let mut word = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || RESERVED_CHARS.contains(next) {
                        break;
                    }
                    word.push(chars.next().unwrap());
                    index += 1;
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    Ok(tokens)
}
