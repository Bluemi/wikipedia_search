use std::fmt::{Display, Formatter};
use nom::{branch::alt, IResult};
use nom::bytes::complete::{take, tag, take_until, take_till1, take_while};
use nom::character::complete::{line_ending, multispace0};
use nom::combinator::map;
use nom::multi::{count, many0, many1, many1_count, many_m_n};
use nom::sequence::{delimited, pair, preceded, tuple};

// TODO: replace &nbsp; with space
#[derive(Debug)]
enum Token<'a> {
    Text(&'a str),  // Some word
    Paragraph,     // double newline
    Newline, // single newline
    HtmlTag {
        tag: &'a str,
        content: &'a str,
    },     // "&lt;NAME&gt;"
    BoldText(&'a str),    // "'''"
    ItalicText(&'a str),  // "''"
    Header { // =, ==, ..., ======
        text: &'a str,
        level: u8
    },
    // [[ or ]]
    Link(&'a str),
    // {{ or }}
    Template(&'a str),
    // *
    UnorderedListEntry {
        text: &'a str,
        level: u8,
    },
    // #
    OrderedListEntry {
        text: &'a str,
        level: u8,
    },
    // "&lt;!--" or "--&gt;"
    Comment,
    Redirect,
}

impl Token<'_> {
    fn get_name(&self) -> &'static str {
        match self {
            Token::Text(_) => "Word",
            Token::Paragraph => "Paragraph",
            Token::Newline => "Newline",
            Token::HtmlTag { .. } => "HtmlTag",
            Token::BoldText(_) => "BoldText",
            Token::ItalicText(_) => "ItalicText",
            Token::Header { .. } => "Header",
            Token::Link { .. } => "Link",
            Token::Template { .. } => "Template",
            Token::UnorderedListEntry { .. } => "UnorderedListEntry",
            Token::OrderedListEntry { .. } => "OrderedListEntry",
            Token::Comment { .. } => "Comment",
            Token::Redirect => "Redirect",
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Text(w) => {
                write!(f, "{}", w)
            }
            Token::Paragraph => {
                write!(f, "\n")
            }
            Token::Newline => {
                write!(f, "\n")
            }
            Token::Header { text, level } => {
                write!(f, "{}{}{}", "=".repeat(*level as usize), text, "=".repeat(*level as usize))
            }
            Token::UnorderedListEntry { level, text } => {
                write!(
                    f, "{} {}",
                    "*".repeat(*level as usize),
                    text
                )
            }
            Token::OrderedListEntry { level, text } => {
                write!(
                    f, "{} {}",
                    "1.".repeat(*level as usize),
                    text,
                )
            }
            Token::Comment { .. } => {
                write!(f, "")
            },
            Token::HtmlTag { tag, content } => {
                write!(f, "<{}>{}</{}>", tag, content, tag)
            }
            Token::BoldText(text) => {
                write!(f, "**{}**", text)
            }
            Token::ItalicText(text) => {
                write!(f, "*{}*", text)
            }
            Token::Link(text) => {
                write!(f, "[[{}]]", text)
            }
            Token::Template(text) => {
                write!(f, "{{{{{}}}}}", text)
            }
            Token::Redirect => {
                write!(f, "#REDIRECT")
            }
        }
    }
}

fn parse_bold(input: &str) -> IResult<&str, Token> {
    map(
        delimited(tag("'''"), take_until("'''"), tag("'''")),
        Token::BoldText,
    )(input)
}

fn parse_italic(input: &str) -> IResult<&str, Token> {
    map(
        delimited(tag("''"), take_until("''"), tag("''")),
        Token::ItalicText,
    )(input)
}

fn parse_template(input: &str) -> IResult<&str, Token> {
    map(
        delimited(tag("{{"), take_until("}}"), tag("}}")),
        Token::Template,
    )(input)
}

fn parse_link(input: &str) -> IResult<&str, Token> {
    map(
        delimited(tag("[["), take_until("]]"), tag("]]")),
        Token::Link,
    )(input)
}

fn parse_single_link(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("[")(input)?;
    let (input, content) = take_until("]")(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, Token::Link(content)))
}

fn parse_html(input: &str) -> IResult<&str, Token> {
    let (input, (tag_name, content)) = tuple((
        // Match the opening tag (e.g., <ref>)
        delimited(tag("&lt;"), take_until("&gt;"), tag("&gt;")),
        // Match the content inside the tags
        take_until("&lt;/"),
    ))(input)?;

    // Ensure the closing tag matches the opening tag (e.g., </ref>)
    let (input, _) = preceded(tag("&lt;/"), tag(tag_name))(input)?;
    let (input, _) = tag("&gt;")(input)?;

    Ok((input, Token::HtmlTag { tag: tag_name, content }))
}

fn parse_unordered_list(input: &str) -> IResult<&str, Token> {
    let (input, _) = many1(line_ending)(input)?;
    let (input, level) = many1_count(tag("*"))(input)?;
    let (input, text) = take_until("\n")(input)?;
    Ok((input, Token::UnorderedListEntry { level: level as u8, text: text.trim() }))
}

fn parse_ordered_list(input: &str) -> IResult<&str, Token> {
    let (input, _) = many1(line_ending)(input)?;
    let (input, level) = many1_count(tag("#"))(input)?;
    let (input, text) = take_until("\n")(input)?;
    Ok((input, Token::OrderedListEntry { level: level as u8, text: text.trim() }))
}

fn parse_redirect(input: &str) -> IResult<&str, Token> {
    let (input, _) = many0(line_ending)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("#REDIRECT")(input)?;
    Ok((input, Token::Redirect))
}

fn parse_colon_start(input: &str) -> IResult<&str, Token> {
    let (input, _) = pair(many1(line_ending), many1(tag(":")))(input)?;
    Ok((input, Token::Newline)) // ignore colon start
}

fn parse_semicolon_start(input: &str) -> IResult<&str, Token> {
    let (input, _) = pair(many1(line_ending), many1(tag(";")))(input)?;
    Ok((input, Token::Newline)) // ignore colon start
}

fn parse_header(input: &str) -> IResult<&str, Token> {
    let (input, _) = many1(line_ending)(input)?;
    let (_, level) = many1_count(tag("="))(input)?;

    if level == 0 {
        return Err(nom::Err::Error(nom::error::make_error(input, nom::error::ErrorKind::Fail)));
    }

    // Match the header text between the `=` symbols
    let (input, text) = delimited(
        count(tag("="), level),   // Match the left `=` symbols
        take_while(|c| c != '='), // Match text in between
        count(tag("="), level),   // Match the right `=` symbols
    )(input)?;

    Ok((input, Token::Header { level: level as u8, text: text.trim() }))
}
fn parse_hline(input: &str) -> IResult<&str, Token> {
    let (input, _) = delimited(many1(line_ending), many_m_n(4, 1000, tag("-")), line_ending)(input)?;
    Ok((input, Token::Paragraph))
}

fn parse_paragraph(input: &str) -> IResult<&str, Token> {
    let (input, _) = alt((
        many_m_n(2, 1000, line_ending),
    ))(input)?;
    Ok((input, Token::Paragraph))
}

fn parse_newline(input: &str) -> IResult<&str, Token> {
    let (input, _) = line_ending(input)?;
    Ok((input, Token::Newline))
}

fn special_sign(input: char) -> bool {
    match input {
        '{' => true,
        '[' => true,
        '\'' => true,
        '\n' => true,
        '&' => true,
        _ => false,
    }
}

fn parse_comment(input: &str) -> IResult<&str, Token> {
    let (input, _) = delimited(tag("&lt;!--"), take_until("-->"), tag("-->"))(input)?;
    Ok((input, Token::Comment))
}

fn parse_normal_text(input: &str) -> IResult<&str, Token> {
    alt((
        map(
            take_till1(special_sign),
            Token::Text
        ),
        map(
            take(1u8),
            Token::Text
        )
    ))(input)
}

fn parse_next_token(input: &str) -> IResult<&str, Token> {
    alt((
        parse_redirect,
        parse_bold,
        parse_italic,
        parse_template,
        parse_link,
        parse_single_link,
        parse_header,
        parse_hline,
        parse_unordered_list,
        parse_ordered_list,
        parse_colon_start,
        parse_semicolon_start,
        parse_html,
        parse_paragraph,
        parse_newline,
        parse_comment,
        parse_normal_text,
    ))(input)
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let mut input = text;
    let mut tokens: Vec<Token> = Vec::new();
    while !input.is_empty() {
        match parse_next_token(input) {
            Ok(result) => {
                input = result.0;
                tokens.push(result.1);
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
    }
    Ok(tokens)
}

pub fn process_article(title: &str, data: &[u8]) -> Result<(), String> {
    println!("title: {}", title);
    match std::str::from_utf8(data) {
        Ok(text) => {
            match tokenize(text) {
                Ok(result) => {
                    for token in result {
                        println!("{}: {}", token.get_name(), token);
                    }
                }
                Err(_) => {
                    println!("parse failed");
                }
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn test_parser() {
    // let text = "The string '''Alan Smithee''' is a nice {{ name }}.\n\nThis is a [[link]]\
    // \n=== This is a level 3 header===\n some more text\n----\nNext Section\n\
    // Some text with &lt;ref&gt;html&lt;/ref&gt;\
    // {{ template1 {{ template2 }} }}\n\
    // * List Entry 1
    // * List Entry 2
    // ";

    // let text = "Some text\n* List Entry 1\n*** List Entry 2\n";
    // let text = "Some text\n# List Entry 1\n### List Entry 2\n";
    let text = "Some template {{ that {{ is }} nested }}";

    match tokenize(text) {
        Ok(result) => {
            println!("parse success");
            for token in result {
                println!("{}: {}", token.get_name(), token);
            }
        }
        Err(err) => {
            println!("parse failed: {}", err);
        }
    }
}
