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

    pub fn read_line(&mut self) -> Option<&[u8]> {
        loop {
            for i in self.read_pos..self.write_pos {
                if self.buffer[i] == b'\n' {
                    let line = &self.buffer[self.read_pos..i];
                    self.read_pos = i + 1;
                    return Some(line);
                }
            }

            if self.read_pos > 0 {
                let remaining = self.write_pos - self.read_pos;
                if remaining > 0 {
                    self.buffer.copy_within(self.read_pos..self.write_pos, 0);
                }
                self.write_pos = remaining;
                self.read_pos = 0;
            }

            if self.write_pos == self.buffer.len() {
                self.write_pos = 0;
                panic!("line too long");
            }

            let chunk = &mut self.buffer[self.write_pos..];
            match self
                .stream
                .read(chunk)
                .expect("failed to read from the Hyprland event socket")
            {
                0 => return None,
                n => self.write_pos += n,
            }
        }
    }
}
