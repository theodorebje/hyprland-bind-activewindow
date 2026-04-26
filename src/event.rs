use crate::{bufstream::BufStream, instance::Instance};
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
        let mut buffered = BufStream::new(stream);

        while let Some(line_bytes) = buffered.read_line() {
            let line = core::str::from_utf8(line_bytes).unwrap();
            if let Some(stripped) = line.strip_prefix("activewindow>>")
                && let Some((class, title)) = stripped.split_once(',')
            {
                self.0(ActiveWindowChangedEvent {
                    class,
                    _title: title,
                });
            }
        }
    }
}
