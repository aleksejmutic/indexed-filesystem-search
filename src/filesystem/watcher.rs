use crate::config::loader::load_config;
use crate::database::connection::connect;
use crate::filesystem::listener;
use crate::filesystem::scanner::create_file_entry;
use crate::synchronization::sync::{
    delete_directory_event, delete_file_event, rename_directory_event, sync_file_event,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, event::CreateKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use walkdir::WalkDir;

pub fn watch(path: &Path) -> notify::Result<()> {
    // adding exclusions to be taken into consideration
    let exclusions = load_config().expect("Failed to load configuration");

    let conn = connect().expect("Failed to connect to database");

    // unwrapping what channel returns, that would be a transmitter and a receiver
    let (transmiter, receiver) = channel();

    let mut renamed_from: Option<PathBuf> = None; //was None before which is not corrent, a path buffer is expected, PathBuf

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
        // redundant as of now
        // println!("Received event: {:?}", event.kind);

        for path in &event.paths {
            // exclusions are not printed to the console
            if exclusions.should_skip_file(path) {
                continue;
            }

            println!("Path: {}", path.display());

            // event kind for folder creation
            if event.kind == notify::EventKind::Create(CreateKind::Folder) {
                if exclusions.should_skip_directory(path) {
                    continue;
                }

                if let Err(error) = watcher.watch(path, RecursiveMode::NonRecursive) {
                    println!("Failed to watch {}: {:?}", path.display(), error);
                }
            }

            // added an event kind, which is file creation
            if event.kind == notify::EventKind::Create(CreateKind::File) {
                match create_file_entry(path) {
                    Ok(entry) => match sync_file_event(&conn, &entry) {
                        Ok(result) => println!("File synchronization result: {:?}", result),
                        Err(error) => println!("Failed to synchronize file: {:?}", error),
                    },
                    Err(error) => println!(
                        "Failed to create FileEntry for {}: {:?}",
                        path.display(),
                        error
                    ),
                }
            }

            // added event handling for data changes, file content changes, that changes files metadata so it is important to check
            if matches!(
                event.kind,
                notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
            ) {
                match create_file_entry(path) {
                    Ok(entry) => match sync_file_event(&conn, &entry) {
                        Ok(result) => println!("File synchronization result: {:?}", result),
                        Err(error) => println!("Failed to synchronize file: {:?}", error),
                    },
                    Err(error) => println!(
                        "Failed to create FileEntry for {}: {:?}",
                        path.display(),
                        error
                    ),
                }
            }

            // file name modification
            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::To,
                ))
            {
                if let Some(old_path) = renamed_from.take() {
                    if let Err(error) = rename_directory_event(
                        &conn,
                        &old_path.to_string_lossy(),
                        &path.to_string_lossy(),
                    ) {
                        println!("Failed to rename directory in database: {:?}", error);
                    }
                }

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

            // event for renaming the file, which changes the path as well
            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::From,
                ))
            {
                renamed_from = Some(path.to_path_buf());
            }

            // file deletion
            if event.kind == notify::EventKind::Remove(notify::event::RemoveKind::File) {
                match delete_file_event(&conn, &path.to_string_lossy()) {
                    Ok(true) => println!("File deletion result: Deleted"),
                    Ok(false) => println!("File deletion result: Not found"),
                    Err(error) => println!("Failed to delete file: {:?}", error),
                }
            }

            // directory deletion
            if event.kind == notify::EventKind::Remove(notify::event::RemoveKind::Folder) {
                match delete_directory_event(&conn, &path.to_string_lossy()) {
                    Ok(()) => println!("Directory deletion result: Deleted"),
                    Err(error) => println!("Failed to delete directory from database: {:?}", error),
                }
            }

            // added to test to check whether moving or renaming from a path is registered
            // if event.kind
            //     == notify::EventKind::Modify(notify::event::ModifyKind::Name(
            //         notify::event::RenameMode::From,
            //     ))
            // {
            //     if let Err(error) = watcher.unwatch(path) {
            //         println!("Failed to unwatch {}: {:?}", path.display(), error);
            //     }
            // }
        }
    }

    Ok(())
}
