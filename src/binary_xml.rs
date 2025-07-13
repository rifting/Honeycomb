use crate::{ATTRIBUTE, COMMENT, DOCDECL, IGNORABLE_WHITESPACE, PROCESSING_INSTRUCTION};
use crate::{AbxError, PROTOCOL_MAGIC_VERSION_0, Result};
use crate::{CDSECT, END_DOCUMENT, END_TAG, ENTITY_REF, START_DOCUMENT, START_TAG, TEXT};
use crate::{TYPE_BOOLEAN_FALSE, TYPE_BOOLEAN_TRUE};
use crate::{TYPE_BYTES_BASE64, TYPE_BYTES_HEX, TYPE_STRING, TYPE_STRING_INTERNED};
use crate::{TYPE_DOUBLE, TYPE_FLOAT, TYPE_INT, TYPE_INT_HEX, TYPE_LONG, TYPE_LONG_HEX};
use crate::Policy;
use base64::Engine;
use hex;
use std::io::{Read, Seek, SeekFrom, Write};

/// Fast data input reader for binary ABX format
pub struct FastDataInput<R: Read + Seek> {
    reader: R,
    interned_strings: Vec<String>,
}

impl<R: Read + Seek> FastDataInput<R> {
    /// Create a new FastDataInput reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            interned_strings: Vec::new(),
        }
    }

    /// Read a single byte
    pub fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.reader
            .read_exact(&mut buf)
            .map_err(|_| AbxError::ReadError("byte".to_string()))?;
        Ok(buf[0])
    }

    /// Read a 16-bit unsigned integer (big-endian)
    pub fn read_short(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader
            .read_exact(&mut buf)
            .map_err(|_| AbxError::ReadError("short".to_string()))?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Read a 32-bit signed integer (big-endian)
    pub fn read_int(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.reader
            .read_exact(&mut buf)
            .map_err(|_| AbxError::ReadError("int".to_string()))?;
        Ok(i32::from_be_bytes(buf))
    }

    /// Read a 64-bit signed integer (big-endian)
    pub fn read_long(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.reader
            .read_exact(&mut buf)
            .map_err(|_| AbxError::ReadError("long".to_string()))?;
        Ok(i64::from_be_bytes(buf))
    }

    /// Read a 32-bit float
    pub fn read_float(&mut self) -> Result<f32> {
        let int_value = self.read_int()? as u32;
        Ok(f32::from_bits(int_value))
    }

    /// Read a 64-bit double
    pub fn read_double(&mut self) -> Result<f64> {
        let int_value = self.read_long()? as u64;
        Ok(f64::from_bits(int_value))
    }

    /// Read a UTF-8 string
    pub fn read_utf(&mut self) -> Result<String> {
        let length = self.read_short()?;
        let mut buffer = vec![0u8; length as usize];
        self.reader
            .read_exact(&mut buffer)
            .map_err(|_| AbxError::ReadError("UTF string".to_string()))?;
        String::from_utf8(buffer)
            .map_err(|_| AbxError::ReadError("UTF string (invalid UTF-8)".to_string()))
    }

    /// Read an interned UTF-8 string
    pub fn read_interned_utf(&mut self) -> Result<String> {
        let index = self.read_short()?;
        if index == 0xFFFF {
            let string = self.read_utf()?;
            self.interned_strings.push(string.clone());
            Ok(string)
        } else {
            self.interned_strings
                .get(index as usize)
                .cloned()
                .ok_or(AbxError::InvalidInternedStringIndex(index))
        }
    }

    /// Read a byte array of specified length
    pub fn read_bytes(&mut self, length: u16) -> Result<Vec<u8>> {
        let mut data = vec![0u8; length as usize];
        self.reader
            .read_exact(&mut data)
            .map_err(|_| AbxError::ReadError("bytes".to_string()))?;
        Ok(data)
    }

    /// Get current position in the stream
    pub fn tell(&mut self) -> Result<u64> {
        self.reader.stream_position().map_err(AbxError::Io)
    }

    /// Seek to a specific position in the stream
    pub fn seek(&mut self, pos: u64) -> Result<()> {
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    /// Check if we've reached the end of the stream
    pub fn is_eof(&mut self) -> bool {
        let current_pos = match self.reader.stream_position() {
            Ok(pos) => pos,
            Err(_) => return true,
        };

        let end_pos = match self.reader.seek(SeekFrom::End(0)) {
            Ok(pos) => pos,
            Err(_) => return true,
        };

        let _ = self.reader.seek(SeekFrom::Start(current_pos));
        current_pos >= end_pos
    }

    /// Get the interned strings table (for debugging)
    pub fn interned_strings(&self) -> &[String] {
        &self.interned_strings
    }
}

/// XML entity encoder for safe XML output
pub fn encode_xml_entities(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Binary XML deserializer that converts ABX format to XML
pub struct BinaryXmlDeserializer<R: Read + Seek, W: Write> {
    input: FastDataInput<R>,
    output: W,
    collect_policies: bool,
    policies: Vec<Policy>,
    restriction_node_offset: u64,
    already_read_restrictions_user: bool
}

impl<R: Read + Seek, W: Write> BinaryXmlDeserializer<R, W> {
    /// Create a new deserializer with the given reader and writer
    pub fn new(mut reader: R, output: W, collect_policies: bool) -> Result<Self> {
        // Check magic header
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|_| AbxError::ReadError("magic header".to_string()))?;

        if magic != PROTOCOL_MAGIC_VERSION_0 {
            return Err(AbxError::InvalidMagicHeader {
                expected: PROTOCOL_MAGIC_VERSION_0,
                actual: magic,
            });
        }

        Ok(Self {
            input: FastDataInput::new(reader),
            output,
            collect_policies,
            policies: Vec::new(),
            restriction_node_offset: 0,
            already_read_restrictions_user: false
        })
    }

    /// Deserialize the binary XML to text XML
    pub fn deserialize(&mut self) -> Result<()> {
        write!(self.output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;

        while !self.input.is_eof() {
            match self.process_token() {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Error parsing token: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
    /// Process a single token from the binary stream
    fn process_token(&mut self) -> Result<bool> {
        let token = self.input.read_byte()?;
        let command = token & 0x0F;
        let type_info = token & 0xF0;

        match command {
            START_DOCUMENT => Ok(true),

            END_DOCUMENT => Ok(false),

            START_TAG => {
                let tag_name = self.input.read_interned_utf()?;

                if tag_name == "restrictions_user" {
                    self.already_read_restrictions_user = true;
                }

                if tag_name == "restrictions" && self.already_read_restrictions_user {
                    self.restriction_node_offset = self.input.tell()?;
                }

                write!(self.output, "<{}", tag_name)?;

                // Process attributes
                while let Ok(pos) = self.input.tell() {
                    match self.input.read_byte() {
                        Ok(next_token) => {
                            if (next_token & 0x0F) == ATTRIBUTE {
                                self.process_attribute(next_token)?;
                            } else {
                                self.input.seek(pos)?;
                                break;
                            }
                        }
                        Err(_) => {
                            self.input.seek(pos)?;
                            break;
                        }
                    }
                }

                write!(self.output, ">")?;
                Ok(true)
            }

            END_TAG => {
                let tag_name = self.input.read_interned_utf()?;
                write!(self.output, "</{}>", tag_name)?;
                Ok(true)
            }

            TEXT => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    if !text.is_empty() {
                        write!(self.output, "{}", encode_xml_entities(&text))?;
                    }
                }
                Ok(true)
            }

            CDSECT => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "<![CDATA[{}]]>", text)?;
                }
                Ok(true)
            }

            COMMENT => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "<!--{}-->", text)?;
                }
                Ok(true)
            }

            PROCESSING_INSTRUCTION => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "<?{}?>", text)?;
                }
                Ok(true)
            }

            DOCDECL => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "<!DOCTYPE {}>", text)?;
                }
                Ok(true)
            }

            ENTITY_REF => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "&{};", text)?;
                }
                Ok(true)
            }

            IGNORABLE_WHITESPACE => {
                if type_info == TYPE_STRING {
                    let text = self.input.read_utf()?;
                    write!(self.output, "{}", text)?;
                }
                Ok(true)
            }

            _ => {
                eprintln!("Warning: Unknown token: {}", command);
                Ok(true)
            }
        }
    }

    /// Process an attribute token
    fn process_attribute(&mut self, token: u8) -> Result<()> {
        let start_offset = self.input.tell()? as u32 - 1;
        let type_info = token & 0xF0;
        let name = self.input.read_interned_utf()?;
        write!(self.output, " {}=\"", name)?;

        match type_info {
            TYPE_STRING => {
                let value = self.input.read_utf()?;
                write!(self.output, "{}", encode_xml_entities(&value))?;
            }
            TYPE_STRING_INTERNED => {
                let value = self.input.read_interned_utf()?;
                write!(self.output, "{}", encode_xml_entities(&value))?;
            }
            TYPE_INT => {
                let value = self.input.read_int()?;
                write!(self.output, "{}", value)?;
            }
            TYPE_INT_HEX => {
                let value = self.input.read_int()?;
                write!(self.output, "0x{:X}", value)?;
            }
            TYPE_LONG => {
                let value = self.input.read_long()?;
                write!(self.output, "{}", value)?;
            }
            TYPE_LONG_HEX => {
                let value = self.input.read_long()?;
                write!(self.output, "0x{:X}", value)?;
            }
            TYPE_FLOAT => {
                let value = self.input.read_float()?;
                write!(self.output, "{}", value)?;
            }
            TYPE_DOUBLE => {
                let value = self.input.read_double()?;
                write!(self.output, "{}", value)?;
            }
            TYPE_BOOLEAN_TRUE => {
                write!(self.output, "true")?;
            }
            TYPE_BOOLEAN_FALSE => {
                write!(self.output, "false")?;
            }
            TYPE_BYTES_HEX => {
                let length = self.input.read_short()?;
                let bytes = self.input.read_bytes(length)?;
                write!(self.output, "{}", hex::encode_upper(&bytes))?;
            }
            TYPE_BYTES_BASE64 => {
                let length = self.input.read_short()?;
                let bytes = self.input.read_bytes(length)?;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                write!(self.output, "{}", encoded)?;
            }
            _ => {
                return Err(AbxError::UnknownAttributeType(type_info));
            }
        }

        let end_offset = self.input.tell()? as u32;

        if self.collect_policies {
            self.policies.push(Policy {
                name,
                start_offset,
                end_offset,
            });
            // println!("{:?}", self.policies);
        }

        write!(self.output, "\"")?;
        Ok(())
    }

    pub fn get_policies(&self) -> &[Policy] {
        &self.policies
    }

    pub fn get_restriction_node_offset(&self) -> &u64 {
        &self.restriction_node_offset
    }
}
