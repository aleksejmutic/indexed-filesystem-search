use std::path::Path;

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

//a function that returns some of my default exclusions regarding the tree walk, these should be custom added
pub fn default_exclusions() -> Exclusions {
    Exclusions {
        directories: vec![
            ".git".to_string(),
            ".svn".to_string(),
            ".hg".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            "__pycache__".to_string(),
            ".pytest_cache".to_string(),
            ".mypy_cache".to_string(),
            ".ruff_cache".to_string(),
            ".tox".to_string(),
            ".nox".to_string(),
            ".venv".to_string(),
            "venv".to_string(),
            "env".to_string(),
            ".gradle".to_string(),
            ".m2".to_string(),
            ".nuget".to_string(),
            ".cache".to_string(),
            ".npm".to_string(),
            ".yarn".to_string(),
            ".pnpm-store".to_string(),
            ".cargo".to_string(),
            ".rustup".to_string(),
            "dist".to_string(),
            "build".to_string(),
            "out".to_string(),
            "bin".to_string(),
            "obj".to_string(),
            "coverage".to_string(),
            "htmlcov".to_string(),
            ".next".to_string(),
            ".nuxt".to_string(),
            ".svelte-kit".to_string(),
            ".angular".to_string(),
            "vendor".to_string(),
        ],

        file_extensions: vec![
            "pyc".to_string(),
            "pyo".to_string(),
            "pyd".to_string(),
            "class".to_string(),
            "o".to_string(),
            "obj".to_string(),
            "a".to_string(),
            "la".to_string(),
            "tmp".to_string(),
            "temp".to_string(),
            "bak".to_string(),
            "swp".to_string(),
            "swo".to_string(),
            "cache".to_string(),
            "log".to_string(),
            "d".to_string(),
        ],

        files: vec![],
    }
}
