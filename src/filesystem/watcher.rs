use crate::config::loader::load_config;
use crate::database::connection::connect;
use crate::filesystem::listener;
use crate::filesystem::scanner::create_file_entry;
use crate::synchronization::sync::{
    delete_directory_event, delete_file_event, rename_directory_event, rename_file_event,
    sync_file_event,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, event::CreateKind};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use walkdir::WalkDir;

// Registers a whole directory tree: every directory gets a watch, and (optionally) every file is indexed.
// WalkDir yields a directory before its contents, so the watch on a directory is always placed
// BEFORE its children are read. Anything created after the watch produces an event, anything
// created before it is picked up by this walk. Duplicates are harmless because sync_file_event is idempotent.
fn register_tree(
    root: &Path,
    watcher: &mut RecommendedWatcher,
    conn: &Connection,
    skip_directory: &dyn Fn(&Path) -> bool,
    index_files: bool,
) {
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !skip_directory(entry.path()))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                println!("Walk error: {:?}", error);
                continue;
            }
        };

        if entry.file_type().is_dir() {
            // watch first
            if let Err(error) = watcher.watch(entry.path(), RecursiveMode::NonRecursive) {
                println!("Failed to watch {}: {:?}", entry.path().display(), error);
            }
        } else if index_files && entry.file_type().is_file() {
            // then index, the parent directory is already being watched at this point
            match create_file_entry(entry.path()) {
                Ok(file_entry) => {
                    if let Err(error) = sync_file_event(conn, &file_entry) {
                        println!("Failed to synchronize file: {:?}", error);
                    }
                }
                Err(error) => println!(
                    "Failed to create FileEntry for {}: {:?}",
                    entry.path().display(),
                    error
                ),
            }
        }
    }
}

pub fn watch(path: &Path) -> notify::Result<()> {
    // adding exclusions to be taken into consideration
    let exclusions = load_config().expect("Failed to load configuration");

    let conn = connect().expect("Failed to connect to database");

    // unwrapping what channel returns, that would be a transmitter and a receiver
    let (transmiter, receiver) = channel();

    // holds the old path between the From and To halves of a rename
    let mut renamed_from: Option<PathBuf> = None;

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) => listener::listen(event, &transmiter),
            Err(error) => println!("Watcher error: {:?}", error),
        },
        // this will ignore entering symlinks targets
        Config::default().with_follow_symlinks(false),
    )?;

    // initial registration: only watches, files are already covered by the full scan
    register_tree(
        path,
        &mut watcher,
        &conn,
        &|p| exclusions.should_skip_directory(p),
        false,
    );

    println!("Watching: {}", path.display());

    // reading events from the receiver and showing their kind, also printing events and paths
    // VERY IMPORTANT: whenever a directory appears (creation, or move/rename into the tree), it is
    // watched first and only then scanned, so nothing created inside it can be missed
    for event in receiver {
        for path in &event.paths {
            // exclusions are not printed to the console
            if exclusions.should_skip_file(path) {
                continue;
            }

            println!("Path: {}", path.display());

            // folder creation: watch first, then scan what may already exist inside
            if event.kind == notify::EventKind::Create(CreateKind::Folder) {
                if exclusions.should_skip_directory(path) {
                    continue;
                }

                register_tree(
                    path,
                    &mut watcher,
                    &conn,
                    &|p| exclusions.should_skip_directory(p),
                    true,
                );
            }

            // file creation
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

            // file content changes, which also change the file's metadata
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

            // rename/move target. Whether it is a directory is decided here, because the old path
            // no longer exists on disk by the time the From event is handled
            if event.kind
                == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::To,
                ))
            {
                if let Some(old_path) = renamed_from.take() {
                    println!("OLD: {}", old_path.display());
                    println!("NEW: {}", path.display());

                    let result = if path.is_dir() {
                        rename_directory_event(
                            &conn,
                            &old_path.to_string_lossy(),
                            &path.to_string_lossy(),
                        )
                    } else {
                        rename_file_event(
                            &conn,
                            &old_path.to_string_lossy(),
                            &path.to_string_lossy(),
                        )
                    };
                    if let Err(error) = result {
                        println!("Failed to rename in database: {:?}", error);
                    }
                }

                if exclusions.should_skip_directory(path) {
                    continue;
                }

                // watch the new location first, then index whatever is inside. For an in-tree rename
                // the rows were just renamed so this reports Unchanged, for a directory moved in
                // from outside the tree this is what indexes its contents
                register_tree(
                    path,
                    &mut watcher,
                    &conn,
                    &|p| exclusions.should_skip_directory(p),
                    true,
                );
            }

            // rename source, only remember the old path
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
        }
    }

    Ok(())
}
