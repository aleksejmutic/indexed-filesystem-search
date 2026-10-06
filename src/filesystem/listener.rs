use notify::Event;
// multiple producers and since consumer communication channel is being created, very simple as of now
pub fn listen(rx: std::sync::mpsc::Receiver<notify::Result<Event>>) {
    for result in rx {
        match result {
            Ok(event) => {
                println!("Event: {:?}", event.kind);
            }

            Err(error) => {
                println!("Watcher error: {:?}", error);
            }
        }
    }
}
