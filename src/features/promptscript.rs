use std::io::Cursor;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crate::crossterm::QueueableCommand;

use crate::core::cmdoutput::CmdOutput;
use crate::core::core::ShellState;
use crate::eval::eval::eval_expr;
use crate::parser::expand::expand_variable;
use crate::parser::tokenizer::parse_identifier;

/// Represents a parsed token from the prompt script input.
enum Token {
    /// Plain text content.
    Text(String),
    /// A tag with optional key-value attributes (e.g., `[color=red]`).
    Tag { name: String, value: Option<String> },
    /// An end tag for a previously opened tag (e.g., `[/color]`).
    EndTag(String),
    /// A variable placeholder (e.g., `$VAR` or `$?`).
    Variable(String),
}

/// Parses a prompt script into a vector of `Token`s.
///
/// # Arguments
/// - `input`: The prompt script string to tokenize.
///
/// # Returns
/// - A vector of `Token`s representing the parsed structure of the script.
fn tokenize_ps(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut index = 0;

    while let Some(c) = chars.next() {
        match c {
            '[' => {
                // Parse a closing tag or opening tag
                if chars.peek() == Some(&'/') {
                    chars.next(); // Consume '/'
                    index += 1;
                    let tag_name: String = chars.by_ref().take_while(|&c| c != ']').collect();
                    tokens.push(Token::EndTag(tag_name));
                } else {
                    let tag_body: String = chars.by_ref().take_while(|&c| c != ']').collect();
                    if let Some(eq_pos) = tag_body.find('=') {
                        let name = tag_body[..eq_pos].to_string();
                        let value = Some(tag_body[eq_pos + 1..].to_string());
                        tokens.push(Token::Tag { name, value });
                    } else {
                        tokens.push(Token::Tag { name: tag_body, value: None });
                    }
                }
            }
            '$' => {
                // Parse a variable
                if chars.peek() == Some(&'?') {
                    chars.next();
                    tokens.push(Token::Variable("?".to_string()));
                } else {
                    let var_name: String = parse_identifier(&mut chars, &mut index);
                    tokens.push(Token::Variable(var_name));
                }
            }
            _ => {
                // Collect plain text
                let mut text = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next == '[' || next == '$' {
                        break;
                    }
                    text.push(chars.next().unwrap());
                    index += 1;
                }
                tokens.push(Token::Text(text));
            }
        }
    }

    tokens
}

/// Converts a color string into a `crossterm::style::Color`.
///
/// # Arguments
/// - `color`: The string representation of the color (e.g., `#FF0000`, `yellow`).
///
/// # Returns
/// - A `Color` enum corresponding to the input string. Defaults to white if unrecognized.
fn parse_color(color: &str) -> Color {
    if color.starts_with('#') && color.len() == 7 {
        let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
        Color::Rgb { r, g, b }
    } else {
        match color {
            "yellow" => Color::Yellow,
            "red" => Color::Red,
            "blue" => Color::Blue,
            _ => Color::White,
        }
    }
}

/// Renders a list of `Token`s into a command output.
///
/// # Arguments
/// - `state`: The current shell state, used for variable expansion and expression evaluation.
/// - `tokens`: The vector of tokens to render.
///
/// # Returns
/// - A `CmdOutput` containing the rendered prompt script output.
fn render_ps_tokens(state: &mut ShellState, tokens: &[Token]) -> CmdOutput {
    let mut output = CmdOutput::new();
    let mut cursor = Cursor::new(&mut output.stdout);

    for token in tokens {
        match token {
            Token::Text(text) => {
                cursor.queue(Print(text)).unwrap();
            }
            Token::Variable(var_name) => {
                cursor.queue(Print(expand_variable(state, var_name))).unwrap();
            }
            Token::Tag { name, value } => {
                match name.as_str() {
                    "color" => {
                        if let Some(v) = value {
                            cursor.queue(SetForegroundColor(parse_color(v))).unwrap();
                        }
                    }
                    "cmd" => {
                        if let Some(expr) = value {
                            eval_expr(state, expr).unwrap();
                        }
                    }
                    _ => (),
                }
            }
            Token::EndTag(tag_name) => {
                if tag_name == "color" {
                    cursor.queue(ResetColor).unwrap();
                }
            }
        }
    }

    output
}

/// Evaluates a prompt script expression.
///
/// # Arguments
/// - `state`: The current shell state, used for variable expansion and expression evaluation.
/// - `expr`: The prompt script expression to evaluate.
///
/// # Returns
/// - A `CmdOutput` containing the evaluation result.
pub fn eval_ps(state: &mut ShellState, expr: &str) -> CmdOutput {
    render_ps_tokens(state, &tokenize_ps(expr))
}
