use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::QueueableCommand;

use crate::core::ShellState;
use crate::eval::expand_variable;
use crate::eval::eval_expr;
use crate::tokenizer::parse_variable;

enum Token {
    Text(String),
    Tag { name: String, value: Option<String> },
    EndTag(String),
    Variable(String)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '[' => {
                // Check for a possible closing tag '/'
                if chars.peek() == Some(&'/') {
                    chars.next(); // Consume '/'
                    let tag_name: String = chars.by_ref().take_while(|&c| c != ']').collect();
                    tokens.push(Token::EndTag(tag_name));
                } else {
                    // Parse opening tag (key=value or just word)
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
                let var_name: String = parse_variable(&mut chars);
                tokens.push(Token::Variable(var_name));
            }
            _ => {
                // Collect text until we reach another '[' or '$'
                let mut text = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next == '[' || next == '$' {
                        break;
                    }
                    text.push(chars.next().unwrap());
                }
                tokens.push(Token::Text(text));
            }
        }
    }
    return tokens;
}

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
            _ => Color::White
        }
    }
}

fn print_tokens(state: &mut ShellState, tokens: &[Token]) {
    for token in tokens {
        match token {
            Token::Text(text) => {
                state.stdout.queue(Print(text)).unwrap();
            },
            Token::Variable(var_name) => {
                if let Some(value) = expand_variable(state, var_name) {
                    state.stdout.queue(Print(format!("{}", value))).unwrap();
                } else {
                    state.stdout.queue(Print(format!("${}", var_name))).unwrap();
                }
            }
            Token::Tag { name, value } => {
                match name.as_str() {
                    "color" => {
                        if let Some(v) = value {
                            state.stdout.queue(SetForegroundColor(parse_color(v))).unwrap();
                        }
                    },
                    "cmd" => {
                        if let Some(expr) = value {
                            eval_expr(state, expr).unwrap();
                        }
                    }
                    _ => ()
                }
            }
            Token::EndTag(tag_name) => {
                match tag_name.as_str() {
                    "color" => {
                        state.stdout.queue(ResetColor).unwrap();
                    },
                    _ => ()
                }
            }
        }
    }
}

pub fn print_expr(state: &mut ShellState, expr: &str) {
    print_tokens(state, &tokenize(expr))
}