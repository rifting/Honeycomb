use crate::{BinaryXmlDeserializer, Result, SeekableReader};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, Write};

/// High-level converter for ABX to XML conversion
pub struct AbxToXmlConverter;

impl AbxToXmlConverter {
    /// Convert ABX from a reader to a writer
    ///
    /// This is the most flexible method, allowing conversion between
    /// any types that implement Read+Seek and Write respectively.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    /// use std::fs::File;
    ///
    /// let input = File::open("input.abx").unwrap();
    /// let output = File::create("output.xml").unwrap();
    /// AbxToXmlConverter::convert(input, output).unwrap();
    /// ```
    pub fn convert<R: Read + Seek, W: Write>(reader: R, writer: W) -> Result<()> {
        let mut deserializer = BinaryXmlDeserializer::new(reader, writer, false)?;
        deserializer.deserialize()
    }

    /// Convert ABX file to XML file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// AbxToXmlConverter::convert_file("input.abx", "output.xml").unwrap();
    /// ```
    pub fn convert_file(input_path: &str, output_path: &str) -> Result<()> {
        if input_path == output_path {
            return Self::convert_file_in_place(input_path);
        }

        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);

        let output_file = File::create(output_path)?;
        let writer = BufWriter::new(output_file);

        Self::convert(reader, writer)
    }

    /// Convert ABX from stdin to stdout (streaming with seek capability)
    ///
    /// Uses a SeekableReader to provide seeking capability over stdin.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// // This would be called when processing: cat file.abx | abx2xml - -
    /// AbxToXmlConverter::convert_stdin_stdout().unwrap();
    /// ```
    pub fn convert_stdin_stdout() -> Result<()> {
        let stdin = io::stdin();
        let reader = SeekableReader::new(stdin.lock());
        let stdout = io::stdout();
        let writer = BufWriter::new(stdout.lock());

        Self::convert(reader, writer)
    }

    /// Convert ABX from stdin to file (streaming with seek capability)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// // This would be called when processing: cat file.abx | abx2xml - output.xml
    /// AbxToXmlConverter::convert_stdin_to_file("output.xml").unwrap();
    /// ```
    pub fn convert_stdin_to_file(output_path: &str) -> Result<()> {
        let stdin = io::stdin();
        let reader = SeekableReader::new(stdin.lock());
        let output_file = File::create(output_path)?;
        let writer = BufWriter::new(output_file);

        Self::convert(reader, writer)
    }

    /// Convert ABX file to stdout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// AbxToXmlConverter::convert_file_to_stdout("input.abx").unwrap();
    /// ```
    pub fn convert_file_to_stdout(input_path: &str) -> Result<()> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let writer = io::stdout();

        Self::convert(reader, writer)
    }

    /// Convert ABX file in place (overwrites the original file)
    ///
    /// This method reads the entire file into memory, converts it,
    /// and then writes the result back to the same file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// // This is called internally when using the -i flag
    /// AbxToXmlConverter::convert_file("input.abx", "input.abx").unwrap();
    /// ```
    fn convert_file_in_place(file_path: &str) -> Result<()> {
        // Read entire file into memory
        let input_file = File::open(file_path)?;
        let mut reader = BufReader::new(input_file);
        let mut file_data = Vec::new();
        reader.read_to_end(&mut file_data)?;

        // Convert from memory
        let cursor = Cursor::new(file_data);
        let mut output_data = Vec::new();
        {
            let writer = Cursor::new(&mut output_data);
            Self::convert(cursor, writer)?;
        }

        // Write back to file
        let output_file = File::create(file_path)?;
        let mut writer = BufWriter::new(output_file);
        writer.write_all(&output_data)?;
        writer.flush()?;

        Ok(())
    }

    /// Convert ABX data from a byte slice to a String
    ///
    /// This is a convenience method for converting ABX data that's already in memory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// let abx_data = std::fs::read("input.abx").unwrap();
    /// let xml_string = AbxToXmlConverter::convert_bytes(&abx_data).unwrap();
    /// println!("{}", xml_string);
    /// ```
    pub fn convert_bytes(abx_data: &[u8]) -> Result<String> {
        let cursor = Cursor::new(abx_data);
        let mut output_data = Vec::new();
        {
            let writer = Cursor::new(&mut output_data);
            Self::convert(cursor, writer)?;
        }
        String::from_utf8(output_data)
            .map_err(|_| crate::AbxError::ParseError("Invalid UTF-8 in output".to_string()))
    }

    /// Convert ABX data from a Vec<u8> to a String
    ///
    /// This takes ownership of the input data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use abx2xml::AbxToXmlConverter;
    ///
    /// let abx_data = std::fs::read("input.abx").unwrap();
    /// let xml_string = AbxToXmlConverter::convert_vec(abx_data).unwrap();
    /// println!("{}", xml_string);
    /// ```
    pub fn convert_vec(abx_data: Vec<u8>) -> Result<String> {
        let cursor = Cursor::new(abx_data);
        let mut output_data = Vec::new();
        {
            let writer = Cursor::new(&mut output_data);
            Self::convert(cursor, writer)?;
        }
        String::from_utf8(output_data)
            .map_err(|_| crate::AbxError::ParseError("Invalid UTF-8 in output".to_string()))
    }
}
