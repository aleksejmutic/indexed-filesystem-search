use notify::event::ModifyKind;
use notify::{Event, EventKind};

// multiple producers and since consumer communication channel is what is planned to be created, the listener does different things based on event kind
// as of now events are directly taken
pub fn listen(event: Event) {
    match event.kind {
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_)) => {
            println!("Event: {:?}", event.kind);

            for path in event.paths {
                if path.to_string_lossy().contains("watcher-test") {
                    println!("Event: {:?}", event.kind);
                    println!("Path: {}", path.display());
                }
            }
        }

        _ => {}
    }
}
