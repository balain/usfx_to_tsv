// usfx_to_tsv: Convert USFX XML files to TSV
// Starting point: Reader example at https://crates.io/crates/quick-xml
// Find some USFX files at https://ebible.org/ - e.g. https://ebible.org/find/show.php?id=engnet

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::str;

fn main() {
    let mut reader = Reader::from_file("./xml/source.xml").expect("Couldn't find or open file");
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_verse:bool = false;
    let mut in_word:bool = false;
    let mut start_verse:bool = false;
    let mut in_content:bool = false;
    let mut in_s:bool = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"p" => {},
                    b"s" => {
                        in_s = true;
                    },
                    b"w" => {
                        in_word = true;
                    },
                    b"v" => {
                        in_verse = true;
                    },
                    b"ve" => { in_verse = false; println!("***ve***");  },
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                if in_content {
                    if in_word && !in_s {
                        if start_verse { print!(" "); } // TODO: Fix this hack - figure out spacing
                        print!("{}", e.unescape().unwrap().into_owned());
                    } else {
                        if !in_s { // Don't print section headings
                            print!("{}", str::from_utf8(e.as_ref()).unwrap());
                        }
                    }
                }
            },
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"w" => {
                        in_word = false;
                    },
                    b"s" => {
                        in_s = false;
                    },
                    _ => (),
                }
            },
            Ok(Event::CData(e)) => {
                println!("cdata:{:?}", e);
            },
            Ok(Event::Empty(e)) => {
                if e.name() == quick_xml::name::QName(b"ve") {
                    in_verse = false;
                    println!("");
                } else {
                    let mut attrvec = e.attributes().map(|a| a).collect::<Vec<_>>();
                    for a in attrvec.iter_mut() {
                        let k = str::from_utf8(a.as_ref().unwrap().key.as_ref()).unwrap();
                        let v = str::from_utf8(&a.as_ref().unwrap().value.as_ref()).unwrap().trim();
                        if k.eq("bcv") {
                            let bcv_vec = v.split(".").collect::<Vec<_>>();
                            print!("{:}\t{:}\t{:}\t", bcv_vec[0], bcv_vec[1], bcv_vec[2]);
                            start_verse = true;
                            in_content = true;
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            // There are several other `Event`s we do not consider here
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
}