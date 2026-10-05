use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn watch(path: &Path) -> notify::Result<()> {
    let mut watcher = RecommendedWatcher::new(
        |result: notify::Result<Event>| match result {
            Ok(event) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    println!("Event: {:?}", event.kind);

                    for path in event.paths {
                        println!("Path: {}", path.display());
                    }
                }

                _ => {}
            },

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
