use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)] //needs to be serializable in order to be written and read from toml
pub struct Exclusions {
    pub directories: Vec<String>,
    pub file_extensions: Vec<String>,
    pub files: Vec<String>,
}

impl Exclusions {
    pub fn should_skip_directory(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| self.directories.iter().any(|excluded| excluded == name))
            .unwrap_or(false)
    }

    pub fn should_skip_file_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                self.file_extensions
                    .iter()
                    .any(|excluded| excluded == extension)
            })
            .unwrap_or(false)
    }

    pub fn should_skip_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| self.files.iter().any(|excluded| excluded == name))
            .unwrap_or(false)
    }
}
