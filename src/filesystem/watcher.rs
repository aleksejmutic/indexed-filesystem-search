use crate::config::loader::load_config;
use crate::filesystem::listener;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use walkdir::WalkDir;

pub fn watch(path: &Path) -> notify::Result<()> {
    // adding exclusions to be taken into consideration
    let exclusions = load_config().expect("Failed to load configuration");

    let mut watcher = RecommendedWatcher::new(
        |result: notify::Result<Event>| match result {
            Ok(event) => listener::listen(event),
            Err(error) => println!("Watcher error: {:?}", error),
        },
        // this will ignore entering symlinks targets
        Config::default().with_follow_symlinks(false),
    )?;

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !exclusions.should_skip_directory(entry.path()))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                println!("Walk error: {:?}", error);
                continue;
            }
        };

        if entry.file_type().is_dir() {
            if let Err(error) = watcher.watch(entry.path(), RecursiveMode::NonRecursive) {
                println!("Failed to watch {}: {:?}", entry.path().display(), error);
            }
        }
    }

    println!("Watching: {}", path.display());

    loop {
        std::thread::park();
    }
}
