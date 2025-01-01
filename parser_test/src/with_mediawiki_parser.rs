use mediawiki_parser::{parse, Element};

const MINIMUM_WORDS_PER_TABLE_CELL: usize = 3;

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

pub fn parse_article(text: &String, mut result: &mut Vec<String>) {
    match parse(&text) {
        Ok(elem) => {
            println!("done parsing");
            println!("start formatting article");
            print_element(&elem, &mut result);
            println!("done formatting article");
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

pub fn process_text_mediawiki_parser(title: &str, data: &[u8]) -> usize {
    let text = String::from_utf8_lossy(data);
    let text = text.replace("&lt;", "<").replace("&gt;", ">");
    println!("#########################");
    println!("title: {}", title);
    let mut result = Vec::new();
    println!("start parsing");
    parse_article(&text, &mut result);
    result.len()
    // for r in result {
    //     println!("{}", r);
    // }
    // println!("text: {}", text);
}
