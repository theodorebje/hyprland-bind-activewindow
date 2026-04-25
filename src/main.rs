use hyprland::{event_listener::EventListener, keyword::Keyword};

// kitty, SUPER, q, exec, uwsm app -- kitty
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: &'static str = Box::leak(args.join(" ").into_boxed_str());
    let (class, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (modifiers, rest): (&'static str, &'static str) = rest.split_once(", ").unwrap();
    let (key, action): (&'static str, &'static str) = rest.split_once(", ").unwrap();

    dbg!(class, modifiers, key, action);

    let instance = hyprland::default_instance_panic();
    let mut listener = EventListener::new();
    listener.add_active_window_changed_handler(move |wevent| {
        if let Some(wevent) = wevent {
            if wevent.class == class {
                Keyword::instance_set(instance, "unbind", format!("{modifiers},{key}")).unwrap();
                println!("keyword unbind {modifiers}, {key}");
            } else {
                Keyword::instance_set(instance, "bind", format!("{modifiers},{key},{action}"))
                    .unwrap();
                println!("keyword bind {modifiers}, {key}, {action}");
            }
        }
    });
    listener.start_listener().unwrap();
}
