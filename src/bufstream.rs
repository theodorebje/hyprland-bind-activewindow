use crate::{EVENT_BUFFER_SIZE, unixstream::UnixStream};

pub struct BufStream {
    stream: UnixStream,
    buffer: [u8; EVENT_BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl BufStream {
    pub const fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            buffer: [0; EVENT_BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Read one line (including newline) and return it without the newline.
    /// Returns `None` on EOF.
    pub fn read_line(&mut self) -> Option<&[u8]> {
        loop {
            // Scan for newline in the current buffer
            for i in self.read_pos..self.write_pos {
                if self.buffer[i] == b'\n' {
                    let line = &self.buffer[self.read_pos..i];
                    self.read_pos = i + 1; // consume the newline
                    return Some(line);
                }
            }

            // No newline found – need more data
            if self.read_pos > 0 {
                // Move remaining data to front of buffer
                let remaining = self.write_pos - self.read_pos;
                if remaining > 0 {
                    self.buffer.copy_within(self.read_pos..self.write_pos, 0);
                }
                self.write_pos = remaining;
                self.read_pos = 0;
            }

            // If buffer is full and still no newline, the line is too long.
            if self.write_pos == self.buffer.len() {
                // Handle error: line exceeds buffer size
                self.write_pos = 0; // discard and continue? better to panic or return error
                panic!("line too long");
            }

            // Read more data into the free space
            let chunk = &mut self.buffer[self.write_pos..];
            match self.stream.read(chunk).unwrap() {
                0 => return None, // EOF
                n => self.write_pos += n,
            }
        }
    }
}
