use crate::filesystem::entry::FileEntry;
use crate::filesystem::error::ScanError;
use std::env;
use walkdir::WalkDir;

pub fn scan() -> Result<Vec<FileEntry>, ScanError> {
    let home_directory = match env::home_dir() {
        Some(path) => path,
        None => panic!("Could not determine the home directory."),
    };

    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in WalkDir::new(home_directory) {
        let entry = entry?;
        let metadata = entry.metadata()?;

        let file_entry = FileEntry {
            path: entry.path().to_path_buf(),
            filename: entry.path().file_name().map(|name| name.to_os_string()), //closures, closures, closures...
            extension: entry
                .path()
                .extension()
                .map(|extension| extension.to_os_string()),
            size: metadata.len(),
            modified: metadata.modified()?,
            is_directory: metadata.is_dir(),
            is_hidden: entry
                .path()
                .file_name()
                .map(|name| name.to_string_lossy().starts_with('.'))
                .unwrap_or(false), //if it is None, it just returns a bool false, just unwraps the Option enum with a fallback/default
        };
        println!("{}", entry.path().display());

        entries.push(file_entry);
    }

    Ok(entries)
}
