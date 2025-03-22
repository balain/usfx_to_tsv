//! USFX to TSV Converter
//! 
//! This crate provides functionality to convert USFX (Unified Scripture Format XML) files to TSV format.
//! USFX files can be found at https://ebible.org/ (e.g., https://ebible.org/find/show.php?id=engnet)
//! 
//! # Example
//! ```no_run
//! use usfx_to_tsv::UsfxParser;
//! use std::fs::File;
//! 
//! let config = UsfxConfig::default();
//! let output = Box::new(File::create("output.tsv").unwrap());
//! let mut parser = UsfxParser::new("input.xml", output, config).unwrap();
//! parser.parse().unwrap();
//! ```

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::BufReader;
use std::str;
use std::path::Path;
use std::io::Write;

/// Configuration options for the USFX parser
#[derive(Debug, Clone)]
pub struct UsfxConfig {
    /// Buffer size for XML parsing (default: 1024)
    pub buffer_size: usize,
    /// Whether to trim whitespace from text (default: true)
    pub trim_text: bool,
    /// Whether to include debug output (default: false)
    pub debug_output: bool,
}

impl Default for UsfxConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            trim_text: true,
            debug_output: false,
        }
    }
}

/// Builder for UsfxConfig
#[derive(Debug, Default)]
pub struct UsfxConfigBuilder {
    config: UsfxConfig,
}

impl UsfxConfigBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            config: UsfxConfig::default(),
        }
    }

    /// Set the buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Set whether to trim text
    pub fn trim_text(mut self, trim: bool) -> Self {
        self.config.trim_text = trim;
        self
    }

    /// Set whether to include debug output
    pub fn debug_output(mut self, debug: bool) -> Self {
        self.config.debug_output = debug;
        self
    }

    /// Build the configuration
    pub fn build(self) -> UsfxConfig {
        self.config
    }
}

#[derive(Debug)]
pub enum ParserError {
    FileError(std::io::Error),
    XmlError(quick_xml::Error),
    ParseError(String),
}

#[derive(Debug, PartialEq, Clone)]
enum ParserState {
    Book,
    Initial,
    InVerse,
    InWord,
    InSection,
    InFootnote,
    InCrossReference,
    VerseEnd,
}

/// Main parser for USFX files
pub struct UsfxParser {
    reader: Reader<BufReader<std::fs::File>>,
    state: ParserState,
    buffer: Vec<u8>,
    output: Box<dyn Write>,
    config: UsfxConfig,
}

impl UsfxParser {
    /// Create a new USFX parser
    /// 
    /// # Arguments
    /// * `input_path` - Path to the input USFX file
    /// * `output` - Writer for the output TSV
    /// * `config` - Configuration options for the parser
    /// 
    /// # Returns
    /// * `Result<Self, ParserError>` - The parser instance or an error
    pub fn new<P: AsRef<Path>>(
        input_path: P,
        output: Box<dyn Write>,
        config: UsfxConfig,
    ) -> Result<Self, ParserError> {
        let reader = Reader::from_file(input_path)
            .map_err(|e| ParserError::FileError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        Ok(Self {
            reader,
            state: ParserState::Initial,
            buffer: Vec::with_capacity(config.buffer_size),
            output,
            config,
        })
    }

    /// Parse the USFX file and convert it to TSV format
    /// 
    /// # Returns
    /// * `Result<(), ParserError>` - Success or error
    pub fn parse(&mut self) -> Result<(), ParserError> {
        let mut in_content = false;
        let mut last_state = ParserState::Initial;

        loop {
            match self.reader.read_event_into(&mut self.buffer) {
                Err(e) => return Err(ParserError::XmlError(e)),
                
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"book" => self.state = ParserState::Book,
                        b"ve" => self.state = ParserState::VerseEnd,
                        b"w" => {
                            // Ignore words outside of paragraphs
                            if in_content {
                                self.state = ParserState::InWord
                            }
                        },
                        b"v" => {
                            if in_content {
                                self.state = ParserState::InVerse
                            }
                        },
                        b"s" => {
                            self.state = ParserState::InSection;
                            in_content = false;
                        },
                        b"f" => self.state = ParserState::InFootnote,
                        b"x" => self.state = ParserState::InCrossReference,
                        _ => (),
                    }
                },

                Ok(Event::Text(e)) => {
                    if in_content && self.state != ParserState::InFootnote && self.state != ParserState::InCrossReference && self.state != ParserState::InSection && self.state != ParserState::Book {
                        let text = e.unescape()
                            .map_err(|e| ParserError::ParseError(format!("Failed to unescape text: {}", e)))?
                            .into_owned();
                        
                        let text = if self.config.trim_text {
                            text.trim()
                        } else {
                            &text
                        };
                        // write!(self.output, "[{:?}]", self.state).map_err(|e| ParserError::ParseError(e.to_string()))?;
                        
                        match self.state {
                            ParserState::InVerse => {
                                    match text {
                                        "\n" => write!(self.output, "^").map_err(|e| ParserError::ParseError(e.to_string()))?,
                                        _ => write!(self.output, "{}", text).map_err(|e| ParserError::ParseError(e.to_string()))?,
                                    }
                            },
                            ParserState::InWord => {
                                match last_state {
                                    ParserState::Initial => write!(self.output, "{}", text).map_err(|e| ParserError::ParseError(e.to_string()))?,
                                    ParserState::InWord => /* no op */ (),
                                    _ => write!(self.output, " {}", text).map_err(|e| ParserError::ParseError(e.to_string()))?,
                                }
                            },
                            _ => {
                                // write!(self.output, "{}", text).map_err(|e| ParserError::ParseError(e.to_string()))?;
                            }
                        }
                    }
                    last_state = self.state.clone();

                },

                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"ve" => self.state = ParserState::Initial,
                        b"w" => self.state = ParserState::InVerse,
                        b"v" => self.state = ParserState::InVerse,
                        b"s" => self.state = ParserState::Initial,
                        b"f" => self.state = ParserState::Initial,
                        b"x" => self.state = ParserState::Initial,
                        _ => (),
                    }
                },

                Ok(Event::Empty(e)) => {
                    if e.name() == quick_xml::name::QName(b"ve") {
                        self.state = ParserState::Initial;
                        writeln!(self.output).map_err(|e| ParserError::ParseError(e.to_string()))?;
                    } else if e.name() == quick_xml::name::QName(b"v") {
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| ParserError::ParseError(e.to_string()))?;
                            let key = str::from_utf8(attr.key.as_ref())
                                .map_err(|e| ParserError::ParseError(e.to_string()))?;
                            
                            if key == "bcv" {
                                let value = str::from_utf8(attr.value.as_ref())
                                    .map_err(|e| ParserError::ParseError(e.to_string()))?;
                                
                                let parts: Vec<&str> = value.split('.').collect();
                                if parts.len() == 3 {
                                    write!(self.output, "{}\t{}\t{}\t", parts[0], parts[1], parts[2])
                                        .map_err(|e| ParserError::ParseError(e.to_string()))?;
                                    in_content = true;
                                }
                            }
                        }
                    
                    }
                },

                Ok(Event::Eof) => break,
                _ => (),
            }
            self.buffer.clear();
        }
        Ok(())
    }
}

fn main() -> Result<(), ParserError> {
    let config = UsfxConfigBuilder::new()
        .debug_output(true)
        .build();
    let output = Box::new(std::io::stdout());
    let mut parser = UsfxParser::new("./xml/source.xml", output, config)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_parsing() {        
        todo!()
    }
}