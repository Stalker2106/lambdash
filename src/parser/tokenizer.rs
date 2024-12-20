use std::iter::Peekable;
use unic_emoji_char::is_emoji;
use crate::core::error::StatusEnum;

/// Enumeration representing different types of redirections.
#[derive(Debug, PartialEq, Clone)]
pub enum RedirectionType {
    Input,  // >
    Output, // <
    Append, // >>
    Heredoc // <<
}

/// Enumeration representing logical conditions.
#[derive(Debug, PartialEq, Clone)]
pub enum ConditionType {
    And, // &&
    Or,  // ||
}

/// Enumeration representing the various token types.
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

/// Enumeration for tokenization errors.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenizationError {
    UnmatchedCharacter = 127,
}

impl StatusEnum for TokenizationError {
    fn status(&self) -> u16 {
        *self as u16
    }
}

/// Processes an escape sequence, such as `\n` or `\\`, and returns the escaped content.
///
/// If the current character is a backslash (`\`), this function consumes the backslash
/// and the next character, appending them together as a single sequence.
///
/// # Arguments
///
/// * `iter` - A mutable iterator over the input characters.
/// * `index` - A mutable reference to the current character index.
/// * `peeked` - The current character being processed.
///
/// # Returns
///
/// * `Some(String)` - The escaped sequence, if the current character is `\`.
/// * `None` - If the current character is not a backslash.
fn handle_escaping(
    iter: &mut Peekable<std::str::Chars>,
    index: &mut i32,
    peeked: char,
) -> Option<String> {
    if peeked == '\\' {
        let mut sequence = peeked.to_string();
        iter.next();
        *index += 1;
        if let Some(escaped) = iter.peek() {
            sequence.push(*escaped);
            *index += 1;
            iter.next();
        }
        Some(sequence)
    } else {
        None
    }
}

/// Parses a valid identifier, such as a variable name, from the input.
///
/// An identifier consists of alphanumeric characters, underscores, or emojis.
///
/// # Arguments
///
/// * `iter` - A mutable iterator over the input characters.
/// * `index` - A mutable reference to the current character index.
///
/// # Returns
///
/// A `String` containing the parsed identifier.
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
    identifier
}

const SEPARATOR_CHARS: &str = "\"';|$<>";

/// Parses characters until a separator or whitespace is encountered.
///
/// Handles escaped characters and appends them to the resulting string.
///
/// # Arguments
///
/// * `iter` - A mutable iterator over the input characters.
/// * `index` - A mutable reference to the current character index.
///
/// # Returns
///
/// A `String` containing the parsed characters.
fn parse_until_separator(iter: &mut Peekable<std::str::Chars>, index: &mut i32) -> String {
    let mut word = String::new();
    while let Some(&next) = iter.peek() {
        if let Some(sequence) = handle_escaping(iter, index, next) {
            word.push_str(&sequence);
        } else if next.is_whitespace() || SEPARATOR_CHARS.contains(next) {
            break;
        } else {
            word.push(iter.next().unwrap());
            *index += 1;
        }
    }
    word
}

/// Parses characters until a specified closing character is found.
///
/// If the closing character is not found, an error is returned.
///
/// # Arguments
///
/// * `iter` - A mutable iterator over the input characters.
/// * `index` - A mutable reference to the current character index.
/// * `closing_char` - The character that marks the end of the parsed content.
///
/// # Returns
///
/// * `Ok(String)` - The content between the current position and the closing character.
/// * `Err(TokenizationError)` - If the closing character is not found.
fn parse_until_next(
    iter: &mut Peekable<std::str::Chars>,
    index: &mut i32,
    closing_char: char,
) -> Result<String, TokenizationError> {
    let mut closed = false;
    let mut content = String::new();
    while let Some(&next) = iter.peek() {
        if let Some(sequence) = handle_escaping(iter, index, next) {
            content.push_str(&sequence);
        } else if next == closing_char {
            closed = true;
            iter.next();
            *index += 1;
            break;
        } else {
            content.push(iter.next().unwrap());
        }
    }
    if closed {
        Ok(content)
    } else {
        Err(TokenizationError::UnmatchedCharacter)
    }
}

/// Tokenizes a shell expression into a sequence of tokens.
///
/// Handles various types of tokens, including words, operators, variables, and subexpressions.
///
/// # Arguments
///
/// * `expr` - The input string to tokenize.
///
/// # Returns
///
/// * `Ok(Vec<Token>)` - A vector of tokens if the input is successfully tokenized.
/// * `Err(TokenizationError)` - If an unmatched character is encountered.
///
/// # Examples
///
/// ```
/// let input = String::from("echo $USER && ls | grep txt");
/// let tokens = tokenize(&input).unwrap();
/// assert_eq!(tokens.len(), 7); // Includes "echo", "$USER", "&&", "ls", "|", "grep", "txt"
/// ```
pub fn tokenize(expr: &String) -> Result<Vec<Token>, TokenizationError> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    let mut index = 0;

    while let Some(c) = chars.next() {
        index += 1;
        match c {
            '#' => break, // Ignore comments.
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Operator(ConditionType::Or));
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            '!' => tokens.push(Token::Negate),
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Operator(ConditionType::And));
                } else {
                    tokens.push(Token::Background);
                }
            }
            '>' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection(RedirectionType::Append));
                } else {
                    tokens.push(Token::Redirection(RedirectionType::Output));
                }
            }
            '<' => {
                if chars.peek() == Some(&'<') {
                    chars.next();
                    index += 1;
                    tokens.push(Token::Redirection(RedirectionType::Heredoc));
                } else {
                    tokens.push(Token::Redirection(RedirectionType::Input));
                }
            }
            '$' => tokens.push(Token::Variable(parse_identifier(&mut chars, &mut index))),
            '\'' | '"' => match parse_until_next(&mut chars, &mut index, c) {
                Ok(content) => tokens.push(Token::Word(content)),
                Err(error) => return Err(error),
            },
            '`' | '(' => match parse_until_next (
                &mut chars,
                &mut index,
                if c == '(' { ')' } else { '`' },
            ) {
                Ok(content) => match tokenize(&content) {
                    Ok(subtokens) => tokens.push(Token::Subexpression(subtokens)),
                    Err(error) => return Err(error),
                },
                Err(error) => return Err(error),
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