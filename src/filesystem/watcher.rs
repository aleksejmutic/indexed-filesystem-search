use crate::config::loader::load_config;
use crate::filesystem::listener;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, event::CreateKind};
use std::path::Path;
use std::sync::mpsc::channel;
use walkdir::WalkDir;

pub fn watch(path: &Path) -> notify::Result<()> {
    // adding exclusions to be taken into consideration
    let exclusions = load_config().expect("Failed to load configuration");

    // unwrapping what channel returns, that would be a transmitter and a receiver
    let (transmiter, receiver) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) => listener::listen(event, &transmiter),
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

    // reading events from the receiver and showing their kind, also printing events and paths
    // VERY IMPORTANT: When an event which is either a creation, or moving of a directory, that directory
    // should automatically be watched and registered by notify, and that is assured here, a NonRecursive walk is done on those directories
    for event in receiver {
        println!("Received event: {:?}", event.kind);

        for path in &event.paths {
            println!("Path: {}", path.display());

            if event.kind == notify::EventKind::Create(CreateKind::Folder) {
                if exclusions.should_skip_directory(path) {
                    continue;
                }

                if let Err(error) = watcher.watch(path, RecursiveMode::NonRecursive) {
                    println!("Failed to watch {}: {:?}", path.display(), error);
                }
            }

            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::To,
                ))
            {
                if exclusions.should_skip_directory(path) {
                    continue;
                }

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
                        if let Err(error) = watcher.watch(entry.path(), RecursiveMode::NonRecursive)
                        {
                            println!("Failed to watch {}: {:?}", entry.path().display(), error);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
