mod parser_wiki_de;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use crate::parser_wiki_de::{tokenize, Token};

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

fn tokens_to_vec(tokens: &Vec<Token>) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current_part = String::new();

    for token in tokens {
        match token {
            Token::Text(text) => {
                current_part.push_str(text);
            }
            Token::Paragraph | Token::Newline => {
                if !current_part.is_empty() {
                    parts.push(current_part);
                    current_part = String::new();
                }
            }
            Token::Header { text, .. } => {
                if !current_part.is_empty() {
                    parts.push(current_part);
                    current_part = String::new();
                }
                if matches!(text.to_lowercase().trim(), "einzelnachweise" | "literatur" | "weblinks") {
                    break;
                }
                parts.push(text.to_string());
            }
            Token::Link(content) => {
                current_part.push_str(content);
            }
            Token::UnorderedListEntry { tokens, .. } => {
                if !current_part.is_empty() {
                    parts.push(current_part);
                    current_part = String::new();
                }
                let sub_parts = tokens_to_vec(tokens);
                parts.extend(sub_parts);
            }
            Token::OrderedListEntry { tokens, .. } => {
                if !current_part.is_empty() {
                    parts.push(current_part);
                    current_part = String::new();
                }
                let sub_parts = tokens_to_vec(tokens);
                parts.extend(sub_parts);
            }
            Token::Redirect => {
                // redirects are not searchable
                return Vec::new();
            }
            Token::Ignore | Token::HtmlTag {..} | Token::HtmlSign {..} | Token::Template(_) | Token::Table(_) | Token::Comment => {},
        }
    }
    if !current_part.is_empty() {
        parts.push(current_part);
    }
    parts
}

#[pyfunction]
fn parse_wiki(text: &str) -> PyResult<Vec<String>> {
    match tokenize(text) {
        Ok(result) => {
            Ok(tokens_to_vec(&result))
        }
        Err(_) => {
            Err(PyValueError::new_err("Failed to tokenize text"))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn wiki_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(parse_wiki, m)?)?;
    Ok(())
}
