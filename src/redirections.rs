use crate::{expression::Redirection, fsio::{open_file, read_file_as_input, write_output_to_file, FSError}, tokenizer::RedirectionType};


pub fn handle_input_redirections(redirections: &Vec<Redirection>) -> Result<Option<Vec<u8>>, FSError> {
  for (index, redirection) in redirections.iter().enumerate() {
      //Skip all input redirections until last...

      // If this is the last redirection, read and return input
      if index == redirections.len() - 1 {
          match redirection.rtype {
              RedirectionType::Input => match read_file_as_input(&redirection.target) {
                  Ok(input) => return Ok(Some(input)),
                  Err(error) => return Err(error)
              },
              RedirectionType::Heredoc => {
                  return Ok(None)
              }
              _ => ()
          }
      }
  }
  return Ok(None);
}

pub fn handle_output_redirections(redirections: &Vec<Redirection>, output: &Vec<u8>) -> Result<bool, FSError> {
  for (index, redirection) in redirections.iter().enumerate() {
      if index == redirections.len() - 1 {
          if let Err(error) = write_output_to_file(&output, &redirection.target, if redirection.rtype == RedirectionType::Output { false } else { true }) {
              return Err(error);
          }
          return Ok(true);
      } else {
          if let Err(error) = open_file( &redirection.target, if redirection.rtype == RedirectionType::Output { false } else { true }) {
              return Err(error);
          }
      }
  }
  return Ok(false);
}
