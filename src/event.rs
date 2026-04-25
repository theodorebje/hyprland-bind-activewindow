use crate::instance::Instance;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct ActiveWindowChangedEvent<'a> {
    pub class: &'a str,
    pub title: &'a str,
}

pub struct ActiveWindowChangedEventListener<F: Fn(ActiveWindowChangedEvent) + 'static>(pub F);

impl<F: Fn(ActiveWindowChangedEvent) + 'static> ActiveWindowChangedEventListener<F> {
    pub fn start(&self, instance: &Instance) {
        let stream = instance.get_event_stream();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();

            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }

            let line = line.trim_end_matches('\n');

            if let Some(stripped) = line.strip_prefix("activewindow>>") {
                let (class, title) = stripped.split_once(',').unwrap();
                self.0(ActiveWindowChangedEvent { class, title });
            }
        }
    }
}
