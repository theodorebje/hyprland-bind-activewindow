use crate::{EVENT_BUFFER_SIZE, instance::Instance};
use core::str;

#[derive(Debug, Clone, Copy)]
pub struct ActiveWindowChangedEvent<'a> {
    pub class: &'a str,
    pub _title: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveWindowChangedEventListener<F: Fn(ActiveWindowChangedEvent) + 'static>(pub F);

impl<F: Fn(ActiveWindowChangedEvent) + 'static> ActiveWindowChangedEventListener<F> {
    pub fn start(&self, instance: &Instance) {
        let stream = instance.get_event_stream();
        let mut buffer = [0u8; EVENT_BUFFER_SIZE];
        let mut pos = 0;

        loop {
            // Read one byte at a time until newline or buffer full
            let mut byte = 0;
            if stream.read(core::slice::from_mut(&mut byte)).unwrap() == 0 {
                break; // EOF
            }

            if byte == b'\n' {
                // Process the line
                let line = core::str::from_utf8(&buffer[..pos]).unwrap();
                if let Some(stripped) = line.strip_prefix("activewindow>>")
                    && let Some((class, title)) = stripped.split_once(',')
                {
                    self.0(ActiveWindowChangedEvent {
                        class,
                        _title: title,
                    });
                }
                // Reset position for next line
                pos = 0;
            } else if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            } else {
                // Line too long – handle error (skip line, panic, etc.)
                // For simplicity, reset and continue
                pos = 0;
            }
        }
    }
}
