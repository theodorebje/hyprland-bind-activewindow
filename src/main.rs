use hyprland::{
    bind,
    dispatch::DispatchType,
    event_listener::EventListener,
    keyword::Keyword,
};

fn main() {
    let instance = hyprland::default_instance_panic();
    let mut listener = EventListener::new();
    listener.add_active_window_changed_handler(|wevent| {
        if let Some(wevent) = wevent {
            if wevent.class == "kitty" {
                Keyword::instance_set(instance, "unbind", "SUPER,q").unwrap();
                println!("keyword unbind SUPER, q");
            } else {
                bind!(instance, SUPER, Key, "q" => Exec, "uwsm app -- kitty").unwrap();
                println!("keyword bind SUPER, q, exec, uwsm app -- kitty");
            }
        }
    });
    listener.start_listener().unwrap();
}
