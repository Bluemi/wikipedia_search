use std::fmt::{Display, Formatter};
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
enum Token {
    Word(String),  // Some word
    Paragraph,     // double newline
    Space,         // " " or "&nbsp;"
    LowerThan,     // "&lt;"
    GreaterThan,   // > or "&gt;"
    FormatBold,    // "'''"
    FormatItalic,  // "''"
    Header(usize), // =, ==, ..., ======
    // [[ or ]]
    Link { is_start: bool },
    // {{ or }}
    Template { is_start: bool },
    // *
    UnorderedListStart(u8),
    // #
    OrderedListStart(u8),
    // ; or :
    DefinitionListStart { header: bool },
    // "&lt;!--" or "--&gt;"
    Comment { is_start: bool },
    Pipe, // |
}

impl Token {
    fn get_name(&self) -> &'static str {
        match self {
            Token::Word(_) => "Word",
            Token::Paragraph => "Paragraph",
            Token::Space => "Space",
            Token::LowerThan => "LowerThan",
            Token::GreaterThan => "GreaterThan",
            Token::FormatBold => "FormatBold",
            Token::FormatItalic => "FormatItalic",
            Token::Header(_) => "Header",
            Token::Link { .. } => "Link",
            Token::Template { .. } => "Template",
            Token::UnorderedListStart(_) => "UnorderedListStart",
            Token::OrderedListStart(_) => "OrderedListStart",
            Token::DefinitionListStart { .. } => "DefinitionListStart",
            Token::Comment { .. } => "Comment",
            Token::Pipe => "Pipe",
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Word(w) => {
                write!(f, "{}", w)
            }
            Token::Paragraph => {
                write!(f, "\n")
            }
            Token::Space => {
                write!(f, " ")
            }
            Token::LowerThan => {
                write!(f, "<")
            }
            Token::GreaterThan => {
                write!(f, ">")
            }
            Token::Header(level) => {
                write!(f, "{}", "=".repeat(*level))
            }
            Token::Link { is_start } => {
                if *is_start {
                    write!(f, "[[")
                } else {
                    write!(f, "]]")
                }
            }
            Token::Template { is_start } => {
                if *is_start {
                    write!(f, "{{{{")
                } else {
                    write!(f, "}}}}")
                }
            }
            Token::UnorderedListStart(level) => {
                write!(f, "{}", "*".repeat(*level as usize))
            }
            Token::OrderedListStart(level) => {
                write!(f, "{}", "*".repeat(*level as usize))
            }
            Token::DefinitionListStart { header } => {
                if *header {
                    write!(f, ";")
                } else {
                    write!(f, ":")
                }
            }
            Token::Pipe => {
                write!(f, "|")
            }
            Token::Comment { .. } => Ok(()),
            Token::FormatBold => Ok(()),
            Token::FormatItalic => Ok(()),
        }
    }
}

fn matches_string(chars: &[&str], index: usize, target: &str) -> bool {
    for (i, c) in UnicodeSegmentation::graphemes(target, true).enumerate() {
        if index + i >= chars.len() || chars[index + i] != c {
            return false;
        }
    }
    true
}

fn is_start_of_newline(graphemes: &[&str], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    graphemes[index - 1] == "\n"
}

fn num_matching_chars(graphemes: &[&str], index: usize, target: &str) -> usize {
    let mut offset = 0;
    while graphemes[index + offset] == target {
        offset += 1;
    }
    offset
}

fn match_space(graphemes: &[&str], index: usize) -> usize {
    if graphemes[index] == " " { 1 } else { 0 }
}

fn match_paragraph(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "\n\n") {
        2
    } else {
        0
    }
}

fn match_lower_than(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "&lt;") {
        4
    } else {
        0
    }
}

fn match_greater_than(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "&gt;") {
        4
    } else {
        0
    }
}

fn match_format_bold(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "'''") {
        3
    } else {
        0
    }
}

fn match_format_italic(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "''") {
        2
    } else {
        0
    }
}

fn match_header(graphemes: &[&str], index: usize) -> usize {
    num_matching_chars(graphemes, index, "=")
}

fn match_link_start(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "[[") {
        2
    } else {
        0
    }
}

fn match_link_end(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "]]") {
        2
    } else {
        0
    }
}

fn match_template_start(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "{{") {
        2
    } else {
        0
    }
}

fn match_template_end(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "}}") {
        2
    } else {
        0
    }
}

fn match_unordered_list_start(graphemes: &[&str], index: usize) -> usize {
    if is_start_of_newline(graphemes, index) {
        num_matching_chars(graphemes, index, "*")
    } else {
        0
    }
}

fn match_ordered_list_start(graphemes: &[&str], index: usize) -> usize {
    if is_start_of_newline(graphemes, index) {
        num_matching_chars(graphemes, index, "#")
    } else {
        0
    }
}

fn match_definition_list_start_header(graphemes: &[&str], index: usize) -> usize {
    if is_start_of_newline(graphemes, index) {
        (graphemes[index] == ";") as usize
    } else {
        0
    }
}

fn match_definition_list_start_noheader(graphemes: &[&str], index: usize) -> usize {
    if is_start_of_newline(graphemes, index) {
        (graphemes[index] == ":") as usize
    } else {
        0
    }
}

fn match_comment_start(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "&lt;!--") {
        7
    } else {
        0
    }
}

fn match_comment_end(graphemes: &[&str], index: usize) -> usize {
    if matches_string(graphemes, index, "--&gt;") {
        6
    } else {
        0
    }
}

fn match_pipe(graphemes: &[&str], index: usize) -> usize {
    (graphemes[index] == "|") as usize
}

fn special_sign(grapheme: &str, start_of_line: bool) -> bool {
    if start_of_line {
        matches!(
            grapheme,
            " " | "&" | "{" | "}" | "[" | "]" | "'" | "=" | "*" | "#" | ":" | ";" | "-"
        )
    } else {
        matches!(
            grapheme,
            " " | "&" | "{" | "}" | "[" | "]" | "'" | "=" | "*" | "#" | ":" | ";" | "-"
        )
    }
}

fn match_text(graphemes: &[&str], index: usize) -> (String, usize) {
    let mut result = Vec::new();
    let mut i = index;
    while i < graphemes.len() {
        if parse_next_token_except_word(graphemes, i).is_some() {
            break;
        } else {
            result.push(graphemes[i]);
        }
        i += 1;
    }
    (result.join(""), i - index)
}

fn parse_next_token_except_word(graphemes: &[&str], index: usize) -> Option<(Token, usize)> {
    let size = match_comment_start(graphemes, index);
    if size != 0 {
        return Some((Token::Comment { is_start: true }, size));
    }
    let size = match_comment_end(graphemes, index);
    if size != 0 {
        return Some((Token::Comment { is_start: false }, size));
    }
    let size = match_space(graphemes, index);
    if size != 0 {
        return Some((Token::Space, size));
    }
    let size = match_paragraph(graphemes, index);
    if size != 0 {
        return Some((Token::Paragraph, size));
    }
    let size = match_lower_than(graphemes, index);
    if size != 0 {
        return Some((Token::LowerThan, size));
    }
    let size = match_greater_than(graphemes, index);
    if size != 0 {
        return Some((Token::GreaterThan, size));
    }
    let size = match_format_bold(graphemes, index);
    if size != 0 {
        return Some((Token::FormatBold, size));
    }
    let size = match_format_italic(graphemes, index);
    if size != 0 {
        return Some((Token::FormatItalic, size));
    }
    let size = match_header(graphemes, index);
    if size != 0 {
        return Some((Token::Header(size), size));
    }
    let size = match_link_start(graphemes, index);
    if size != 0 {
        return Some((Token::Link { is_start: true }, size));
    }
    let size = match_link_end(graphemes, index);
    if size != 0 {
        return Some((Token::Link { is_start: false }, size));
    }
    let size = match_template_start(graphemes, index);
    if size != 0 {
        return Some((Token::Template { is_start: true }, size));
    }
    let size = match_template_end(graphemes, index);
    if size != 0 {
        return Some((Token::Template { is_start: false }, size));
    }
    let size = match_unordered_list_start(graphemes, index);
    if size != 0 {
        return Some((Token::UnorderedListStart(size as u8), size));
    }
    let size = match_ordered_list_start(graphemes, index);
    if size != 0 {
        return Some((Token::OrderedListStart(size as u8), size));
    }
    let size = match_definition_list_start_header(graphemes, index);
    if size != 0 {
        return Some((Token::DefinitionListStart { header: true }, size));
    }
    let size = match_definition_list_start_noheader(graphemes, index);
    if size != 0 {
        return Some((Token::DefinitionListStart { header: false }, size));
    }
    let size = match_pipe(graphemes, index);
    if size != 0 {
        return Some((Token::Pipe, size));
    }
    None
}

fn parse_next_token(graphemes: &[&str], index: usize) -> Option<(Token, usize)> {
    if let Some((token, offset)) = parse_next_token_except_word(graphemes, index) {
        Some((token, offset))
    } else {
        // if nothing else matches, this is normal text
        let (word, size) = match_text(graphemes, index);
        if size != 0 {
            Some((Token::Word(word), size))
        } else {
            None
        }
    }
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let graphemes = UnicodeSegmentation::graphemes(text, true).collect::<Vec<&str>>();
    let mut index = 0;
    let mut tokens = Vec::new();
    while index < graphemes.len() {
        if let Some((token, offset)) = parse_next_token(&graphemes, index) {
            index += offset;
            println!("token: {}", token.get_name());
            println!("{}", token);
            tokens.push(token);
            std::io::stdout().flush().unwrap();
        }
    }
    Ok(tokens)
}

pub fn process_article(title: &str, data: &[u8]) -> Result<(), String> {
    println!("title: {}", title);
    match std::str::from_utf8(data) {
        Ok(text) => {
            tokenize(text)?;
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
