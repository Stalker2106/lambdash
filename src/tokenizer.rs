use std::iter::Peekable;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

pub enum Token {
    Word(String),
    Pipe,         // |
    Background,   // &
    Redirection(String), // >, <, >>, <<
    Variable(String), // $VAR or ${VAR}
    Operator(String), // && or ||
    EndOfInput,   // End of input
}

const RESERVED_CHARS: &str = "\"';|&$<>";

#[derive(Debug)]
pub struct TokenizationError {
    pub details: String,
    pub status: ExitStatus,
}

impl TokenizationError {
    fn new(code: i32, msg: String) -> TokenizationError {
        TokenizationError{
            status: ExitStatus::from_raw(code),
            details: msg.to_string()
        }
    }
}

pub fn parse_variable(iter: &mut Peekable<std::str::Chars>) -> String {
    let mut var = String::new();
    while let Some(&next) = iter.peek() {
        if next.is_alphanumeric() || next == '_' || (var.len() == 0 && next == '?') {
            var.push(iter.next().unwrap());
        } else {
            break;
        }
    }
    return var;
}

pub fn tokenize(input: &String) -> Result<Vec<Token>, TokenizationError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '|' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::Operator("||".to_string()));
                } else {
                    tokens.push(Token::Pipe);
                }
            },
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::Operator("&&".to_string()));
                } else {
                    tokens.push(Token::Background);
                }
            },
            '>' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Redirection(">>".to_string()));
                } else {
                    tokens.push(Token::Redirection(">".to_string()));
                }
            },
            '<' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Redirection("<<".to_string()));
                } else {
                    tokens.push(Token::Redirection("<".to_string()));
                }
            },
            '$' => {
                tokens.push(Token::Variable(parse_variable(&mut chars)));
            },
            '\'' | '"' => {
                let mut closed = false;
                let quote = c;
                let mut word = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '\'' || next ==  '"' {
                        closed = true;
                        chars.next();
                        break;
                    }
                    word.push(chars.next().unwrap());
                }
                if !closed {
                    return Err(TokenizationError::new(127, format!("Unterminated quote {} found.", quote)));
                }
                tokens.push(Token::Word(word));
            },
            ';' => tokens.push(Token::EndOfInput),
            c if c.is_whitespace() => continue, 
            _ => {
                let mut word = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || RESERVED_CHARS.contains(next) {
                        break;
                    }
                    word.push(chars.next().unwrap());
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    if let Some(_) = tokens.last() {
        tokens.push(Token::EndOfInput);
    }
    Ok(tokens)
}
