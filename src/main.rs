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
    let mut between_word:bool = false;
    let mut ignore:bool = false; // Ignore this block
    let mut start_verse:bool = false;
    let mut in_content:bool = false;
    let mut in_s:bool = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"p" => {
                        // paragraph
                        // no-op
                    },
                    b"s" => {
                        in_s = true;
                        ignore = true;
                    },
                    b"w" => {
                        in_word = true;
                        between_word = false;
                        // print!("<w>");
                    },
                    b"v" => {
                        in_verse = true;
                    },
                    b"ve" => { in_verse = false; println!("***ve***");  },
                    b"f" => { ignore = true; },
                    b"x" => { ignore = true; },
                    _ => (),
                }
            }
            Ok(Event::Text(e)) => {
                if in_content && !ignore {
                    if !in_s && in_verse {
                        if in_word {
                            if start_verse { print!(" "); } // TODO: Fix this hack - figure out spacing
                            print!(">{}", e.unescape().unwrap().into_owned());
                        } else { // Don't print section headings
                            print!("+{}", str::from_utf8(e.as_ref()).unwrap());
                        }
                    } else {
                        // if between_word {
                            print!("{} ", e.unescape().unwrap().into_owned());
                        // } else {
                            // print!("<!BW>");
                        // }
                    }
                } else {
                    if !ignore {
                        print!("^{:?}", e.unescape().unwrap().into_owned());
                    // } else {
                    //     print!("<ignore: {:?}/>", e.unescape().unwrap().into_owned());
                    }
                }
            },
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"w" => {
                        in_word = false;
                        between_word = true;
                        // print!("</w>");
                    },
                    b"s" => {
                        in_s = false;
                        ignore = false;
                        // print!("&");
                    },
                    b"f" => {
                        ignore = false;
                    },
                    b"s" => {
                        ignore = false;
                    },
                    b"x" => {
                        ignore = false;
                    },
                    _ => (),
                        // print!("*</{:#?}>", str::from_utf8(e.name().as_ref()).unwrap());
                    // },
                }
            },
            Ok(Event::CData(e)) => {
                println!("cdata:{:?}", e);
            },
            Ok(Event::Empty(e)) => {
                if e.name() == quick_xml::name::QName(b"ve") {
                    in_verse = false;
                    println!("");
                } else { // Not verse-end
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
                    // print!("@{:?}", e.name());
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