use std::iter::Peekable;

use unic_emoji_char::is_emoji;

#[derive(Debug, PartialEq, Clone)]
pub enum RedirectionType {
    Input,  // >
    Output, // <
    Append, // >>
    Heredoc // <<
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConditionType {
    And, // &&
    Or,  // ||
}

#[derive(Clone)]
pub enum Token {
    Word(String),
    Pipe,                         // |
    Background,                   // &
    Negate,                       // !
    Subexpression(Vec<Token>),    // () or ``
    Redirection(RedirectionType), // >, <, >>, <<
    Variable(String),             // $VAR or ${VAR}
    Operator(ConditionType),      // && or ||
    CommandSeparator,             // ;
}


#[derive(Debug, PartialEq)]
pub enum TokenizationError {
    UnmatchedCharacter
}

pub fn handle_escaping(iter: &mut Peekable<std::str::Chars>, index: &mut i32, peeked: char) -> Option<String> {
    if peeked == '\\' {
        let mut sequence = peeked.to_string();
        iter.next();
        *index += 1;
        if let Some(escaped) = iter.peek() {
            sequence.push(*escaped);
            *index += 1;
            iter.next();
            return Some(sequence);
        }
        return Some(sequence);
    }
    return None;
}

pub fn parse_identifier(iter: &mut Peekable<std::str::Chars>, index: &mut i32) -> String {
    let mut identifier = String::new();
    while let Some(&next) = iter.peek() {
        if let Some(sequence) = handle_escaping(iter, index, next) {
            identifier.push_str(&sequence);
        } else if next.is_alphanumeric() || next == '_' || is_emoji(next) {
            identifier.push(iter.next().unwrap());
            *index += 1;
        } else {
            break;
        }
    }
    return identifier;
}

const SEPARATOR_CHARS: &str = "\"';|$<>";
fn parse_until_separator(iter: &mut Peekable<std::str::Chars>, index: &mut i32) -> String {
    let mut word = String::new();
    while let Some(&next) = iter.peek() {
        if let Some(sequence) = handle_escaping(iter, index, next) {
            word.push_str(&sequence);
        } else if next.is_whitespace() || SEPARATOR_CHARS.contains(next) {
            break;
        }
        word.push(iter.next().unwrap());
        *index += 1;
    }
    return word;
}

fn parse_until_next(iter: &mut Peekable<std::str::Chars>, index: &mut i32, closing_char: char) -> Result<String, TokenizationError> {
    let mut closed = false;
    let mut content = String::new();
    while let Some(&next) = iter.peek() {
        if let Some(sequence) = handle_escaping(iter, index, next) {
            content.push_str(&sequence);
        } if next == closing_char {
            closed = true;
            iter.next();
            *index += 1;
            break;
        }
        content.push(iter.next().unwrap());
    }
    if !closed {
        return Err(TokenizationError::UnmatchedCharacter);
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
                    tokens.push(Token::Operator(ConditionType::Or));
                } else {
                    tokens.push(Token::Pipe);
                }
            },
            '!' => tokens.push(Token::Negate),
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Operator(ConditionType::And));
                } else {
                    tokens.push(Token::Background);
                }
            },
            '>' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection(RedirectionType::Append));
                } else {
                    tokens.push(Token::Redirection(RedirectionType::Output));
                }
            },
            '<' => {
                if chars.peek() == Some(&'<') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection(RedirectionType::Input));
                } else {
                    tokens.push(Token::Redirection(RedirectionType::Heredoc));
                }
            },
            '$' => {
                tokens.push(Token::Variable(parse_identifier(&mut chars, &mut index)));
            },
            '\'' | '"' => match parse_until_next(&mut chars, &mut index, c) {
                Ok(content) => {
                    if let Some(last) = tokens.last_mut() {
                        if let Token::Word(ref mut last_word) = last {
                            last_word.push_str(&content);
                        }
                    } else {
                        tokens.push(Token::Word(content))
                    }
                },
                Err(error) => return Err(error)
            },
            '`' | '(' => match parse_until_next(&mut chars, &mut index, if c == '(' { ')' } else { c }) {
                Ok(content) => {
                    match tokenize(&content) {
                        Ok(subtokens) => tokens.push(Token::Subexpression(subtokens)),
                        Err(error) => return Err(error)
                    }
                },
                Err(error) => return Err(error)
            },
            ';' => tokens.push(Token::CommandSeparator),
            c if c.is_whitespace() => continue,
            c => {
                let mut word = c.to_string();
                word.push_str(&parse_until_separator(&mut chars, &mut index));
                tokens.push(Token::Word(word));
            }
        }
    }
    Ok(tokens)
}
