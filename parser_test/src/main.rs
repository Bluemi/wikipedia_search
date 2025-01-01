use bzip2::read::MultiBzDecoder;
use std::fs::File;
use std::io::{BufReader, Read};
use quick_xml::Reader;
use quick_xml::events::Event;
use mediawiki_parser;
use mediawiki_parser::{Element, ListItemKind, MWError, MarkupType};

const MINIMUM_WORDS_PER_TABLE_CELL: usize = 3;

fn main() -> std::io::Result<()> {
    // Path to the compressed Wikipedia dump file
    let file_path = "../data/dewiki-latest-pages-articles-multistream1.xml-p1p297012.bz2";
    // let file_path = "../data/dewiki-latest-pages-articles-multistream1.xml-p1p297012";

    // Open the bzip2-compressed file
    let file = File::open(file_path)?;
    let decoder = MultiBzDecoder::new(file);
    let reader = BufReader::new(decoder);

    // Set up the XML parser
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut buf = Vec::new();

    let mut in_title = false;
    let mut in_text = false;
    let mut current_title: Option<String> = None;

    let mut counter = 0;

    // Process the XML file
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"title" => in_title = true,
                    b"text" => in_text = true,
                    _ => {}
                }
            },
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"title" => {
                        in_title = false;
                    },
                    b"text" => in_text = false,
                    _ => {}
                }
            },
            Ok(Event::Text(e)) => {
                if in_text {
                    // println!("Text: {:?}", )
                    // assert!(current_title.is_some());
                    if let Some(title) = &current_title {
                        process_text(&title, e.as_ref());
                        break;
                    } else {
                        // println!("{}", String::from_utf8_lossy(e.as_ref()));
                    }

                }
                if in_title {
                    current_title = Some(e.unescape().unwrap().into_owned());
                    // println!("Title: {:?}", e.unescape().unwrap().into_owned())
                    counter += 1;
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => eprintln!("Error: {:?}", e),
            _ => {}
        }

        // Clear the buffer for the next event
        // println!("Buffer size before clear: {}", buf.len());
        buf.clear();
        if counter > 10 {
            break;
        }
    }

    println!("counter: {}", counter);

    Ok(())
}

/*
fn node_to_text<'a>(node: &Node, data: &'a[u8]) -> Cow<'a, str> {
    match node {
        Node::Bold { end, start } => {
            // println!("start={} end={}", start, end);
            let result = String::from_utf8_lossy(&data[*start..*end]);
            // println!("{}", result);
            result
        },
        Node::Category { .. } => {
            println!("found category");
            Cow::default()
        },
        Node::CharacterEntity { .. } => Cow::default(),
        Node::Comment { .. } => Cow::default(),
        Node::DefinitionList { .. } => Cow::default(),
        Node::EndTag { .. } => Cow::default(),
        Node::ExternalLink { .. } => Cow::default(),
        Node::Heading { .. } => Cow::default(),
        Node::HorizontalDivider { .. } => Cow::default(),
        Node::Image { .. } => Cow::default(),
        Node::Italic { .. } => Cow::default(),
        Node::Link { .. } => Cow::default(),
        Node::MagicWord { .. } => Cow::default(),
        Node::OrderedList { .. } => Cow::default(),
        Node::ParagraphBreak { .. } => Cow::default(),
        Node::Parameter { .. } => Cow::default(),
        Node::Preformatted { .. } => Cow::default(),
        Node::Redirect { .. } => Cow::default(),
        Node::StartTag { .. } => Cow::default(),
        Node::Table { .. } => Cow::default(),
        Node::Tag { .. } => Cow::default(),
        Node::Template { .. } => Cow::default(),
        Node::Text { .. } => Cow::default(),
        Node::UnorderedList { .. } => Cow::default(),
        Node::BoldItalic { .. } => Cow::default(),
    }
}
 */

fn print_element(elem: &Element, result: &mut Vec<String>) {
    match elem {
        Element::Document(doc) => {
            // println!("doc");
            for sub_elem in doc.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Heading(heading) => {
            // caption
            let mut caption = Vec::new();
            for sub_elem in heading.caption.iter() {
                print_element(&sub_elem, &mut caption);
            }
            let caption = caption.join("");
            match caption.trim().to_lowercase().as_str() {
                "einzelnachweise" | "literatur" | "weblinks" => {
                    return;
                }
                _ => {
                    result.push(caption);
                }
            }

            // section content
            for sub_elem in heading.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Text(text) => {
            result.push(text.text.clone());
        }
        Element::Formatted(formatted) => {
            for sub_elem in formatted.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Paragraph(paragraph) => {
            let mut content = Vec::new();
            for sub_elem in paragraph.content.iter() {
                print_element(&sub_elem, &mut content);
            }
            let content = content.join("");
            result.push(content);
        }
        Element::Template(template) => {
            for sub_elem in template.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::TemplateArgument(_) => {}
        Element::InternalReference(reference) => {
            for sub_elem in reference.target.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::ExternalReference(_) => {
            /*
            result.push(String::from("EXTERNAL REFERENCE"));
            for sub_elem in reference.caption.iter() {
                print_element(&sub_elem, result);
            }
             */
        }
        Element::ListItem(list_item) => {
            //println!("list_item");
            let mut content = Vec::new();
            for sub_elem in list_item.content.iter() {
                print_element(&sub_elem, &mut content);
            }
            result.push(content.join(""));
        }
        Element::List(list) => {
            //println!("list");
            for sub_elem in list.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Table(table) => {
            //println!("table");
            for sub_elem in table.caption.iter() {
                print_element(&sub_elem, result);
            }
            for sub_elem in table.rows.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::TableRow(table_row) => {
            //println!("table_row");
            let mut content = Vec::new();
            for sub_elem in table_row.cells.iter() {
                print_element(&sub_elem, &mut content);
            }
            result.push(content.join(""));
        }
        Element::TableCell(table_cell) => {
            //println!("table_cell");
            let mut content = Vec::new();
            for sub_elem in table_cell.content.iter() {
                print_element(&sub_elem, &mut content);
            }
            let content = content.join("");
            if content.trim().split_whitespace().count() > MINIMUM_WORDS_PER_TABLE_CELL {
                result.push(String::from(content));
            }
        }
        Element::Comment(_) => {}
        Element::HtmlTag(table) => {
            //println!("html_tag");
            for sub_elem in table.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Gallery(table) => {
            //println!("gallery");
            for sub_elem in table.content.iter() {
                print_element(&sub_elem, result);
            }
        }
        Element::Error(_) => {
            println!("error");
        }
    }

}

fn process_text(title: &str, data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let text = text.replace("&lt;", "<").replace("&gt;", ">");
    println!("#########################");
    println!("title: {}", title);
    let mut result = Vec::new();
    match mediawiki_parser::parse(&text) {
        Ok(elem) => {
            print_element(&elem, &mut result);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    for r in result {
        println!("{}", r);
    }
    // println!("text: {}", text);
}

