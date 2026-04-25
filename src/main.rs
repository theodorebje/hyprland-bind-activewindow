mod event;
mod instance;

use crate::{event::ActiveWindowChangedEventListener, instance::Instance};
use std::cell::Cell;

// kitty, SUPER, q, exec, uwsm app -- kitty
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: &'static str = Box::leak(args.join(" ").into_boxed_str());
    let (class, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (modifiers, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (key, action): (&'static str, &'static str) = rest.split_once(", ").unwrap();

    dbg!(class, modifiers, key, action);

    let is_bind_set = Cell::new(false);

    let instance = Instance::new();
    let i2 = instance.clone();

    ActiveWindowChangedEventListener(move |wevent| {
        let should_bind_be_set = wevent.class != class;
        if should_bind_be_set == is_bind_set.get() {
            return;
        }
        if should_bind_be_set {
            is_bind_set.set(true);
            i2.set("bind", &format!("{modifiers},{key},{action}"));
            println!("keyword bind {modifiers}, {key}, {action}");
        } else {
            is_bind_set.set(false);
            i2.set("unbind", &format!("{modifiers},{key}"));
            println!("keyword unbind {modifiers}, {key}");
        }
    })
    .start(&instance);
}
