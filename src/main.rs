mod instance;

use crate::instance::Instance;
use hyprland::event_listener::EventListener;

// kitty, SUPER, q, exec, uwsm app -- kitty
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: &'static str = Box::leak(args.join(" ").into_boxed_str());
    let (class, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (modifiers, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (key, action): (&'static str, &'static str) = rest.split_once(", ").unwrap();

    dbg!(class, modifiers, key, action);

    let instance = Instance::new();
    let mut listener = EventListener::new();
    listener.add_active_window_changed_handler(move |wevent| {
        if let Some(wevent) = wevent {
            if wevent.class == class {
                instance.set("unbind", format!("{modifiers},{key}"));
                println!("keyword unbind {modifiers}, {key}");
            } else {
                instance.set("bind", format!("{modifiers},{key},{action}"));
                println!("keyword bind {modifiers}, {key}, {action}");
            }
        }
    });
    listener.start_listener().unwrap();
}
