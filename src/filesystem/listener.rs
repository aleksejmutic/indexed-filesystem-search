use notify::event::ModifyKind;
use notify::{Event, EventKind};
use std::sync::mpsc::Sender;
// multiple producers and since consumer communication channel is what is used inside the watcher
// the important part with the listen() function is taking events by type and passing them to the channel via transmitter
pub fn listen(event: Event, transmitter: &Sender<Event>) {
    match event.kind {
        EventKind::Create(_)
        | EventKind::Remove(_)
        | EventKind::Modify(ModifyKind::Name(_))
        | EventKind::Modify(ModifyKind::Data(_)) => {
            // redundant print
            // println!("Event: {:?}", event.kind);

            // this was just a test to print an exact file to see whether it works in the pipeline
            // for path in &event.paths {
            //     if path.to_string_lossy().contains("watcher-test") {
            //         println!("Event: {:?}", event.kind);
            //         println!("Path: {}", path.display());
            //     }
            // }

            transmitter
                .send(event)
                .expect("Failed to send filesystem event");
        }

        _ => {}
    }
}
