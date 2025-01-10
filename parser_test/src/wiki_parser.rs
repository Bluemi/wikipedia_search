use std::fmt::{Display, Formatter};
use nom::{branch::alt, IResult};
use nom::bytes::complete::{take, tag, take_until, take_till1, take_while, is_not, take_till};
use nom::character::complete::{line_ending, multispace0};
use nom::combinator::map;
use nom::Err::Error;
use nom::error::Error as NomError;
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
    HtmlSign {
        sign: &'a str,
    },
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
        tokens: Vec<Token<'a>>,
        level: u8,
    },
    // #
    OrderedListEntry {
        tokens: Vec<Token<'a>>,
        level: u8,
    },
    // "&lt;!--" or "--&gt;"
    Comment,
    Redirect,
    Ignore,
}

impl Token<'_> {
    fn get_name(&self) -> &'static str {
        match self {
            Token::Text(_) => "Word",
            Token::Paragraph => "Paragraph",
            Token::Newline => "Newline",
            Token::HtmlTag { .. } => "HtmlTag",
            Token::HtmlSign { .. } => "HtmlSign",
            Token::Header { .. } => "Header",
            Token::Link { .. } => "Link",
            Token::Template (_) => "Template",
            Token::UnorderedListEntry { .. } => "UnorderedListEntry",
            Token::OrderedListEntry { .. } => "OrderedListEntry",
            Token::Comment { .. } => "Comment",
            Token::Redirect => "Redirect",
            Token::Ignore => "Ignore",
        }
    }

    fn get_plain_text<'a>(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Text(text) => {
                write!(f, "{}", text)
            },
            Token::Paragraph => {
                write!(f, "\n")
            },
            Token::Newline => {
                write!(f, "\n")
            },
            Token::HtmlTag { tag, content } => {
                write!(f, "<{}>{}</{}>", tag, content, tag)
            }
            Token::HtmlSign { sign } => {
                write!(f, "&{};", sign)
            }
            Token::Header { text, .. } => {
                write!(f, "\n{}\n", text.to_uppercase())
            }
            Token::Link(content) => {
                write!(f, "{}", content)
            }
            Token::Template(name) => {
                write!(f, "{}", name)
            }
            Token::UnorderedListEntry { level, tokens } => {
                write!(f, "{} ", "*".repeat(*level as usize))?;
                for t in tokens {
                    t.get_plain_text(f)?;
                }
                write!(f, "\n")
            }
            Token::OrderedListEntry { level, tokens } => {
                write!(f, "{} ", "#".repeat(*level as usize))?;
                for t in tokens {
                    t.get_plain_text(f)?;
                }
                write!(f, "\n")
            }
            Token::Comment => {
                write!(f, "")
            }
            Token::Redirect => {
                write!(f, "")
            }
            Token::Ignore => {
                write!(f, "")
            }
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
            Token::UnorderedListEntry { level, tokens: content } => {
                write!(
                    f, "{} ",
                    "*".repeat(*level as usize),
                )?;
                for t in content {
                    write!(f, "{}", t)?;
                }
                write!(f, "\n")
            }
            Token::OrderedListEntry { level, tokens } => {
                write!(
                    f, "{} ",
                    "1.".repeat(*level as usize),
                )?;
                for t in tokens {
                    write!(f, "{}", t)?;
                }
                write!(f, "\n")
            }
            Token::Comment { .. } => {
                write!(f, "")
            },
            Token::HtmlTag { tag, content } => {
                write!(f, "<{}>{}</{}>", tag, content, tag)
            },
            Token::HtmlSign { sign } => {
                write!(f, "&{};", sign)
            }
            Token::Link(text) => {
                write!(f, "[[{}]]", text)
            }
            Token::Template(template) => {
                write!(f, "{{{{{}}}}}", template)
            }
            Token::Redirect => {
                write!(f, "#REDIRECT")
            }
            Token::Ignore => {
                write!(f, "")
            }
        }
    }
}

fn parse_bold_italic(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("'''''")(input)?;
    Ok((input, Token::Ignore))
}

fn parse_bold(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("'''")(input)?;
    Ok((input, Token::Ignore))
}

fn parse_italic(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("''")(input)?;
    Ok((input, Token::Ignore))
}

fn parse_template(input: &str) -> IResult<&str, Token> {
    let (input, _) = tag("{{")(input)?;
    let (input, template_name) = parse_template_inner(input)?;
    Ok((input, Token::Template(template_name)))
}

// this just returns the name of the template
fn parse_template_inner(input: &str) -> IResult<&str, &str> {
    let (_, name) = take_till(|c| "}|".contains(c))(input)?;
    let mut running_input = input;
    let mut level = 1;
    loop {
        let (input, _) = take_while(|c| !"{}|".contains(c))(running_input)?;
        if let Ok((input, _)) = tag::<&str, &str, NomError<_>>("{{")(input) {
            level += 1;
            running_input = input;
        } else if let Ok((input, _)) = tag::<&str, &str, NomError<_>>("}}")(input) {
            level -= 1;
            running_input = input;
            if level == 0 {
                break;
            }
        } else {
            let (input, _) = take(1usize)(input)?;
            running_input = input;
        }
    }
    Ok((running_input, name.trim()))
}

fn parse_link(input: &str) -> IResult<&str, Token> {
    let (input, content) = delimited(tag("[["), take_until("]]"), tag("]]"))(input)?;

    let content = content.split("|").last().unwrap();

    Ok((input, Token::Link(content)))
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

    let tag_name = tag_name.split_whitespace().next().unwrap();

    // Ensure the closing tag matches the opening tag (e.g., </ref>)
    let (input, _) = preceded(tag("&lt;/"), tag(tag_name))(input)?;
    let (input, _) = tag("&gt;")(input)?;

    Ok((input, Token::HtmlTag { tag: tag_name, content }))
}

fn parse_html_sign(input: &str) -> IResult<&str, Token> {
    if let Ok((input, _)) = tag::<_, _, NomError<_>>("&amp;nbsp;")(input) {
        return Ok((input, Token::HtmlSign { sign: " " }));
    }
    let (input, sign) = delimited(tag("&"), take_until(";"), tag(";"))(input)?;
    Ok((input, Token::HtmlSign { sign }))
}

fn parse_unordered_list(input: &str) -> IResult<&str, Token> {
    let (input, _) = many1(line_ending)(input)?;
    let (input, level) = many1_count(tag("*"))(input)?;
    let (input, tokens) = parse_until(input, "\n")?;
    // let (input, text) = take_until("\n")(input)?;
    Ok((input, Token::UnorderedListEntry { level: level as u8, tokens }))
}

fn parse_ordered_list(input: &str) -> IResult<&str, Token> {
    let (input, _) = many1(line_ending)(input)?;
    let (input, level) = many1_count(tag("#"))(input)?;
    // let (input, text) = take_until("\n")(input)?;
    let (input, tokens) = parse_until(input, "\n")?;
    Ok((input, Token::OrderedListEntry { level: level as u8, tokens }))
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
        return Err(Error(nom::error::make_error(input, nom::error::ErrorKind::Fail)));
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
    let (input, _) = delimited(tag("&lt;!--"), take_until("--&gt;"), tag("--&gt;"))(input)?;
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

fn parse_until<'a>(input: &'a str, end_tag: &'a str) -> IResult<&'a str, Vec<Token<'a>>> {
    let end_tag = tag::<_, _, NomError<_>>(end_tag);
    let mut input = input;
    let mut tokens = Vec::new();
    while !input.is_empty() {
        let (cur_input, token) = parse_next_token(input)?;
        tokens.push(token);
        input = cur_input;
        if end_tag(input).is_ok() {
            break;
        }
    }
    Ok((input, tokens))
}

fn parse_next_token(input: &str) -> IResult<&str, Token> {
    alt((
        parse_comment,
        parse_redirect,
        parse_bold_italic,
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
        parse_html_sign,
        parse_paragraph,
        parse_newline,
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
                    print_tokens(&result);
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
            print_tokens(&result);
        }
        Err(err) => {
            println!("parse failed: {}", err);
        }
    }
}

fn print_tokens(result: &Vec<Token>) {
    for token in result {
        match token {
            Token::Paragraph | Token::Newline => {
                println!();
            }
            Token::Redirect | Token::Comment | Token::Ignore => {}
            Token::UnorderedListEntry { tokens, .. }  => {
                println!();
                println!("{}: ", token.get_name());
                print_tokens(tokens);
            }
            t => {
                println!("{}: {}", t.get_name(), t);
            }
        }
    }
}
