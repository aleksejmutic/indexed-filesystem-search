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
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

// How long an unmatched From is kept before it is treated as a move out of the watched tree.
// This only affects how long a stale row can linger, not correctness: a late To is handled as a
// move-in and re-indexed by register_tree.
const MOVE_OUT_GRACE: Duration = Duration::from_millis(500);

// cookie (inotify tracker) -> (old path, when the From arrived)
type PendingRenames = HashMap<usize, (PathBuf, Instant)>;

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

// a path was moved out of the watched tree (its To never arrived):
// remove it, and everything under it if it was a directory
fn drop_moved_out(conn: &Connection, old_path: &Path) {
    println!("MOVED OUT: {}", old_path.display());
    if let Err(error) = delete_directory_event(conn, &old_path.to_string_lossy()) {
        println!("Failed to remove moved-out path from database: {:?}", error);
    }
}

// treats every From that has waited longer than the grace period as a move-out
fn sweep_expired(pending: &mut PendingRenames, conn: &Connection) {
    let expired: Vec<usize> = pending
        .iter()
        .filter(|(_, (_, since))| since.elapsed() >= MOVE_OUT_GRACE)
        .map(|(cookie, _)| *cookie)
        .collect();

    for cookie in expired {
        if let Some((old_path, _)) = pending.remove(&cookie) {
            drop_moved_out(conn, &old_path);
        }
    }
}

pub fn watch(path: &Path) -> notify::Result<()> {
    // adding exclusions to be taken into consideration
    let exclusions = load_config().expect("Failed to load configuration");

    let conn = connect().expect("Failed to connect to database");

    // unwrapping what channel returns, that would be a transmitter and a receiver
    let (transmiter, receiver) = channel();

    // From events waiting for their To, paired by the inotify cookie
    let mut pending: PendingRenames = HashMap::new();

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

    // VERY IMPORTANT: whenever a directory appears (creation, or move/rename into the tree), it is
    // watched first and only then scanned, so nothing created inside it can be missed
    loop {
        // nothing pending: sleep with zero CPU until an event arrives.
        // something pending: wake up at most every grace period to expire stale Froms.
        let event = if pending.is_empty() {
            match receiver.recv() {
                Ok(event) => event,
                Err(_) => break,
            }
        } else {
            match receiver.recv_timeout(MOVE_OUT_GRACE) {
                Ok(event) => event,
                Err(RecvTimeoutError::Timeout) => {
                    sweep_expired(&mut pending, &conn);
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        };

        let is_rename_to = event.kind
            == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::To,
            ));
        let is_rename_from = event.kind
            == notify::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::From,
            ));

        // the cookie that pairs the From and To halves of one rename
        let cookie = event.tracker().unwrap_or(0);

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
            if is_rename_to {
                // a matching cookie means an in-tree rename; no match means a move INTO the tree
                if let Some((old_path, _)) = pending.remove(&cookie) {
                    println!("OLD: {}", old_path.display());
                    println!("NEW: {}", path.display());

                    let is_dir = path.is_dir();

                    let result = if is_dir {
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
                    } else if !is_dir {
                        // rename_file_entry only changes the path, so refresh filename, extension,
                        // size, mtime and inode from disk. This also covers atomic saves (write tmp,
                        // rename over target) where no tmp row existed to be renamed.
                        match create_file_entry(path) {
                            Ok(entry) => {
                                if let Err(error) = sync_file_event(&conn, &entry) {
                                    println!("Failed to refresh renamed file: {:?}", error);
                                }
                            }
                            Err(error) => println!(
                                "Failed to create FileEntry for {}: {:?}",
                                path.display(),
                                error
                            ),
                        }
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

            // rename source, remember the old path under its cookie until the To arrives
            if is_rename_from {
                pending.insert(cookie, (path.to_path_buf(), Instant::now()));
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

        // sweep AFTER handling the event, so a To that arrives right at the deadline still
        // finds its From, and a busy event stream can't starve the sweep
        sweep_expired(&mut pending, &conn);
    }

    Ok(())
}
