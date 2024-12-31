use bzip2::read::MultiBzDecoder;
use std::fs::File;
use std::io::BufReader;
use quick_xml::Reader;
use quick_xml::events::Event;
use parse_wiki_text::{Configuration, ConfigurationSource, Node};

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
                    b"title" => in_title = false,
                    b"text" => in_text = false,
                    _ => {}
                }
            },
            Ok(Event::Text(e)) => {
                if in_text {
                    // println!("Text: {:?}", )
                    assert!(current_title.is_some());
                    if let Some(title) = &current_title {
                        let text = String::from_utf8_lossy(e.as_ref());
                        process_text(&title, &text);
                        break;
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
    }

    println!("counter: {}", counter);

    Ok(())
}

fn node_to_text(node: &Node) -> String {
    match node {
        Node::Bold { .. } => {}
        Node::Category { .. } => {}
        Node::CharacterEntity { .. } => {}
        Node::Comment { .. } => {}
        Node::DefinitionList { .. } => {}
        Node::EndTag { .. } => {}
        Node::ExternalLink { .. } => {}
        Node::Heading { .. } => {}
        Node::HorizontalDivider { .. } => {}
        Node::Image { .. } => {}
        Node::Italic { .. } => {}
        Node::Link { .. } => {}
        Node::MagicWord { .. } => {}
        Node::OrderedList { .. } => {}
        Node::ParagraphBreak { .. } => {}
        Node::Parameter { .. } => {}
        Node::Preformatted { .. } => {}
        Node::Redirect { .. } => {}
        Node::StartTag { .. } => {}
        Node::Table { .. } => {}
        Node::Tag { .. } => {}
        Node::Template { .. } => {}
        Node::Text { .. } => {}
        Node::UnorderedList { .. } => {}
        Node::BoldItalic { .. } => {}
    }
}

fn process_text(title: &str, text: &str) {
    // println!("{}", text);

    let result = Configuration::default().parse(text);
    assert!(result.warnings.is_empty());
    for node in result.nodes {
        let part = node_to_text(&node);
    }
}

