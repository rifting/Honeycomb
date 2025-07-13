use std::io::{self, Read, Seek, SeekFrom};

/// A wrapper that provides limited seeking capability by buffering data
///
/// This is useful for converting streams (like stdin) that don't naturally
/// support seeking into seekable readers by buffering the data in memory.
pub struct SeekableReader<R: Read> {
    inner: R,
    buffer: Vec<u8>,
    position: usize,
    end_reached: bool,
}

impl<R: Read> SeekableReader<R> {
    /// Create a new seekable reader wrapping the given reader
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            position: 0,
            end_reached: false,
        }
    }

    /// Get the current position in the buffer
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if we've reached the end of the underlying stream
    pub fn is_end_reached(&self) -> bool {
        self.end_reached
    }

    /// Get the current buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

impl<R: Read> Read for SeekableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we need to read beyond what's currently buffered, read more data
        while self.position + buf.len() > self.buffer.len() && !self.end_reached {
            let mut temp_buf = vec![0u8; 8192]; // Read in 8KB chunks
            match self.inner.read(&mut temp_buf) {
                Ok(0) => {
                    self.end_reached = true;
                    break;
                }
                Ok(n) => {
                    temp_buf.truncate(n);
                    self.buffer.extend_from_slice(&temp_buf);
                }
                Err(e) => return Err(e),
            }
        }

        // Copy data from buffer to output buffer
        let available = self.buffer.len().saturating_sub(self.position);
        let to_copy = buf.len().min(available);

        if to_copy > 0 {
            buf[..to_copy].copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
            self.position += to_copy;
        }

        Ok(to_copy)
    }
}

impl<R: Read> Seek for SeekableReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(pos) => {
                let pos = pos as usize;

                // If seeking beyond current buffer, read more data
                while pos > self.buffer.len() && !self.end_reached {
                    let mut temp_buf = vec![0u8; 8192];
                    match self.inner.read(&mut temp_buf) {
                        Ok(0) => {
                            self.end_reached = true;
                            break;
                        }
                        Ok(n) => {
                            temp_buf.truncate(n);
                            self.buffer.extend_from_slice(&temp_buf);
                        }
                        Err(e) => return Err(e),
                    }
                }

                self.position = pos.min(self.buffer.len());
                Ok(self.position as u64)
            }
            SeekFrom::Current(offset) => {
                let new_pos = if offset >= 0 {
                    self.position.saturating_add(offset as usize)
                } else {
                    self.position.saturating_sub((-offset) as usize)
                };
                self.seek(SeekFrom::Start(new_pos as u64))
            }
            SeekFrom::End(_) => {
                // Read all remaining data to find the end
                while !self.end_reached {
                    let mut temp_buf = vec![0u8; 8192];
                    match self.inner.read(&mut temp_buf) {
                        Ok(0) => {
                            self.end_reached = true;
                            break;
                        }
                        Ok(n) => {
                            temp_buf.truncate(n);
                            self.buffer.extend_from_slice(&temp_buf);
                        }
                        Err(e) => return Err(e),
                    }
                }
                self.position = self.buffer.len();
                Ok(self.position as u64)
            }
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.position as u64)
    }
}
