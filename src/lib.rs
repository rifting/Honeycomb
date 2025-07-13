//! A library for converting Android Binary XML (ABX) to human-readable XML.
//!
//! provides functionality to parse and convert Android Binary XML
//! format to standard XML format. It supports both streaming and file-based conversion.
//!
//! # Examples
//!
//! ```no_run
//! use abx2xml::AbxToXmlConverter;
//! use std::fs::File;
//!
//! // Convert a file
//! AbxToXmlConverter::convert_file("input.abx", "output.xml").unwrap();
//!
//! // Convert from reader to writer
//! let input = File::open("input.abx").unwrap();
//! let output = File::create("output.xml").unwrap();
//! AbxToXmlConverter::convert(input, output).unwrap();
//! ```

use std::io;
use thiserror::Error;

mod binary_xml;
pub mod cli;
mod converter;
mod seekable_reader;

pub use binary_xml::{BinaryXmlDeserializer, FastDataInput, encode_xml_entities};
pub use converter::AbxToXmlConverter;
pub use seekable_reader::SeekableReader;

/// Error types for ABX parsing and conversion
#[derive(Error, Debug)]
pub enum AbxError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error(
        "Invalid ABX file format - magic header mismatch. Expected: {expected:02X?}, got: {actual:02X?}"
    )]
    InvalidMagicHeader { expected: [u8; 4], actual: [u8; 4] },
    #[error("Failed to read {0} from stream")]
    ReadError(String),
    #[error("Invalid interned string index: {0}")]
    InvalidInternedStringIndex(u16),
    #[error("Unknown attribute type: {0}")]
    UnknownAttributeType(u8),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, AbxError>;

// Protocol constants - exposed for advanced users
pub const PROTOCOL_MAGIC_VERSION_0: [u8; 4] = [0x41, 0x42, 0x58, 0x00];

// Command tokens
pub const START_DOCUMENT: u8 = 0;
pub const END_DOCUMENT: u8 = 1;
pub const START_TAG: u8 = 2;
pub const END_TAG: u8 = 3;
pub const TEXT: u8 = 4;
pub const CDSECT: u8 = 5;
pub const ENTITY_REF: u8 = 6;
pub const IGNORABLE_WHITESPACE: u8 = 7;
pub const PROCESSING_INSTRUCTION: u8 = 8;
pub const COMMENT: u8 = 9;
pub const DOCDECL: u8 = 10;
pub const ATTRIBUTE: u8 = 15;

// Type tokens
pub const TYPE_STRING: u8 = 2 << 4;
pub const TYPE_STRING_INTERNED: u8 = 3 << 4;
pub const TYPE_BYTES_HEX: u8 = 4 << 4;
pub const TYPE_BYTES_BASE64: u8 = 5 << 4;
pub const TYPE_INT: u8 = 6 << 4;
pub const TYPE_INT_HEX: u8 = 7 << 4;
pub const TYPE_LONG: u8 = 8 << 4;
pub const TYPE_LONG_HEX: u8 = 9 << 4;
pub const TYPE_FLOAT: u8 = 10 << 4;
pub const TYPE_DOUBLE: u8 = 11 << 4;
pub const TYPE_BOOLEAN_TRUE: u8 = 12 << 4;
pub const TYPE_BOOLEAN_FALSE: u8 = 13 << 4;

#[derive(Debug, Clone)]
pub struct Policy {
    pub name: String,
    pub start_offset: u32,
    pub end_offset: u32
}