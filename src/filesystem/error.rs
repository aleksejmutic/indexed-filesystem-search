pub enum ScanError {
    WalkDir(walkdir::Error),
    Io(std::io::Error),
}

impl From<walkdir::Error> for ScanError {
    fn from(error: walkdir::Error) -> Self {
        ScanError::WalkDir(error)
    }
}

impl From<std::io::Error> for ScanError {
    fn from(error: std::io::Error) -> Self {
        ScanError::Io(error)
    }
}
