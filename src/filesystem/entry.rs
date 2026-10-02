use std::path::PathBuf;
use std::time::SystemTime;

pub struct FileEntry {
    pub path: PathBuf,
    pub filename: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: SystemTime,
    pub is_directory: bool,
    pub is_hidden: bool,
}
