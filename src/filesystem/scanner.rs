use crate::filesystem::entry::FileEntry;
use crate::filesystem::error::ScanError;
use crate::filesystem::exclusions::Exclusions;
use std::env;
use std::os::unix::fs::MetadataExt; //Unix extension trait used to identify the device and inode of a specific file
use walkdir::WalkDir;

pub fn scan(exclusions: &Exclusions) -> Result<Vec<FileEntry>, ScanError> {
    let home_directory = match env::home_dir() {
        Some(path) => path,
        None => panic!("Could not determine the home directory."),
    };

    let mut entries: Vec<FileEntry> = Vec::new();

    let walker = WalkDir::new(home_directory) //prunes the tree, filters the exclusions, so during the tree walk the contents of excluded directories wont be searched
        .into_iter()
        .filter_entry(|entry| !exclusions.should_skip_directory(entry.path()));

    //tree walk
    for entry in walker {
        let entry = entry?;
        let metadata = entry.metadata()?; //tells me what the path points to
        let symlink_metadata = std::fs::symlink_metadata(entry.path())?; //tells me about the path itself without following a symlink

        let file_entry = FileEntry {
            device: metadata.dev(),
            inode: metadata.ino(),
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
            is_symlink: symlink_metadata.file_type().is_symlink(),
            is_executable: metadata.mode() & 0o111 != 0, //this is an octal representation, as that is how permission notation works in Unix
                                                         //111 just means execute bit for owner, group and others
        };
        println!("{}", entry.path().display());

        entries.push(file_entry);
    }

    Ok(entries)
}
