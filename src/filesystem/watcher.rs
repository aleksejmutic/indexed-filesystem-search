use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn watch(path: &Path) -> notify::Result<()> {
    let mut watcher = RecommendedWatcher::new(
        |result: notify::Result<Event>| match result {
            Ok(event) => match event.kind {
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Name(_)) => {
                    println!("Event: {:?}", event.kind);

                    for path in event.paths {
                        if path.to_string_lossy().contains("watcher-test") {
                            println!("Event: {:?}", event.kind);
                            println!("Path: {}", path.display());
                        }
                    }
                }
                _ => {}
            },

            Err(error) => println!("Watcher error: {:?}", error),
        },
        // this will ignore entering symlinks targets
        Config::default().with_follow_symlinks(false),
    )?;

    // let the watcher tolerate eroors, very important to test out!!!
    if let Err(error) = watcher.watch(path, RecursiveMode::Recursive) {
        println!("Failed to watch {}: {:?}", path.display(), error);
    }

    println!("Watching: {}", path.display());

    loop {
        std::thread::park();
    }
}
