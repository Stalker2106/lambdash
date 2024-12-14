use std::vec::Vec;
use std::slice::Iter;
use std::iter::Peekable;

use crate::tokenizer::{ConditionType, RedirectionType, Token};

pub struct Condition {
  operands: (Box<ExpressionTreeNode>, Box<ExpressionTreeNode>),
  ctype: ConditionType
}

pub enum Expression {
  Root,
  Command(Vec<Token>),  // A single command
  Condition(Condition), // && or ||
}

pub struct ExpressionTreeNode {
  expression: Expression,
  dependencies: Vec::<Box<ExpressionTreeNode>>,
  input: Option<Vec<u8>>,
  output: Vec<u8>
}

#[derive(Debug)]
pub enum ExpressionError {
  InvalidPipe,
  InvalidRedirection
}

pub fn build_tree(tokens: Vec<Token>) -> Result<ExpressionTreeNode, ExpressionError> {
  let mut root = ExpressionTreeNode{
    expression: Expression::Root,
    dependencies: Vec::new(),
    input: None,
    output: Vec::new()
  }
  let mut node: Box<ExpressionTreeNode> = Box::new(root);

  let mut token_iter = tokens.iter().peekable();
  while let Some(token) = token_iter.next() {
    node = parse_token(&mut token_iter, &mut node)
}
  return root;
}


pub fn parse_token(prev_token: &mut Peekable<Iter<Token>>, prev_node: &mut Box<ExpressionTreeNode>) -> Result<ExpressionTreeNode, ExpressionError> {
  if let Some(token) = prev_token.peek() {
    match token {
      Token::Word(_) | Token::Variable(_) => {
        match &mut prev_node.expression {
          Expression::Root => {
            let node = ExpressionTreeNode {
              expression: Expression::Command(vec![token]),
              dependencies: Vec::new(),
              input: None,
              output: Vec::new()
            };
            prev_node.dependencies.push(Box::new(node));
            return Ok(node);
          }
          Expression::Command(cmd) => cmd.push(token),
          Expression::Condition(cond) => {
            let node = ExpressionTreeNode {
              expression: Expression::Command(vec![token]),
              dependencies: Vec::new(),
              input: None,
              output: Vec::new()
            };
            cond.operands.1 = Box::new(node);
            return Ok(node);
          }
        }
      },

      Token::Pipe => {
        match prev_node.expression {
          Expression::Root => return Err(ExpressionError::InvalidPipe),
          _ => ()
        }
      },

      // Handle redirection
      Token::Redirection(rtype) => {
        match rtype {
          RedirectionType::Left | RedirectionType::AppendLeft => {
            match prev_node.expression {
              Expression::Command(cmd) => {
                //valid
              },
              _ => return Err(ExpressionError::InvalidRedirection)
            }
          },
          RedirectionType::Right | RedirectionType::AppendRight => {
            match prev_node.expression {
              Expression::Command(cmd) => {
                //Valid
              },
              _ => return Err(ExpressionError::InvalidRedirection)
            }
          }
        }
      },

      // Handle command separator (semicolon)
      Token::CommandSeparator => {
      },

      _ => unimplemented!(),
    }
  }
}