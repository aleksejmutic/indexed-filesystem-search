use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn watch(path: &Path) -> notify::Result<()> {
    let mut watcher = RecommendedWatcher::new(
        |result: notify::Result<Event>| match result {
            Ok(event) => println!("Event: {:?}", event),
            Err(error) => println!("Watcher error: {:?}", error),
        },
        Config::default(),
    )?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    println!("Watching: {}", path.display());

    loop {
        std::thread::park();
    }
}
