use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct FileEntry {
    pub path: PathBuf,
    pub filename: Option<OsString>,
    pub extension: Option<OsString>,
    pub size: u64,
    pub modified: SystemTime,
    pub is_directory: bool,
    pub is_hidden: bool,
}
