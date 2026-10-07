use crate::filesystem::entry::FileEntry;
use crate::filesystem::error::ScanError;
use crate::filesystem::exclusions::Exclusions;
use std::env;
use std::os::unix::fs::MetadataExt; //Unix extension trait used to identify the device and inode of a specific file
use std::path::Path;
use walkdir::WalkDir;

// method for creating a file entry programatically in memory, this is now extracted from the scan function
pub fn create_file_entry(path: &Path) -> Result<FileEntry, ScanError> {
    let metadata = std::fs::metadata(path)?;
    let symlink_metadata = std::fs::symlink_metadata(path)?;

    let file_entry = FileEntry {
        device: metadata.dev(),
        inode: metadata.ino(),
        path: path.to_path_buf(),
        filename: path.file_name().map(|name| name.to_os_string()),
        extension: path.extension().map(|extension| extension.to_os_string()),
        size: metadata.len(),
        modified: metadata.modified()?,
        is_directory: metadata.is_dir(),
        is_hidden: path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false),
        is_symlink: symlink_metadata.file_type().is_symlink(),
        is_executable: !metadata.is_dir() && metadata.mode() & 0o111 != 0,
    };

    Ok(file_entry)
}

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

        // some directory or file can disappearing between WalkDir finding it and itself being created as a file entry in memory,
        // that is why a continue is done instead of raising the error, to avoid program panic
        let file_entry = match create_file_entry(entry.path()) {
            Ok(file_entry) => file_entry,
            Err(error) => {
                println!(
                    "Failed to create FileEntry for {}: {:?}",
                    entry.path().display(),
                    error
                );
                continue; // before this would return an error but then the program would panic and stop working
            }
        };

        println!("{}", entry.path().display());

        entries.push(file_entry);
    }

    Ok(entries)
}
