mod wiki_parser;

use bzip2::read::MultiBzDecoder;
use std::fs::File;
use std::io::BufReader;
use quick_xml::Reader;
use quick_xml::events::Event;
use crate::wiki_parser::{process_article, test_parser};

fn main2() {
    test_parser();
}

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

    // let mut counter = 0;

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
                        // counter += process_text_mediawiki_parser(&title, e.as_ref());
                        if counter == 1 {
                            process_article(title, e.as_ref()).expect("Processing article failed.");
                            break;
                        }
                        counter += 1;
                    } else {
                        // println!("{}", String::from_utf8_lossy(e.as_ref()));
                    }

                }
                if in_title {
                    current_title = Some(e.unescape().unwrap().into_owned());
                    // println!("Title: {:?}", e.unescape().unwrap().into_owned())
                    // counter += 1;
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => eprintln!("Error: {:?}", e),
            _ => {}
        }

        // Clear the buffer for the next event
        // println!("Buffer size before clear: {}", buf.len());
        buf.clear();
        /*
        if counter > 10 {
            break;
        }
         */
    }

    // println!("counter: {}", counter);

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

