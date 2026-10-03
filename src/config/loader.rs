use crate::filesystem::exclusions::Exclusions;
use std::fs;

//function that loads the configuration fro the toml file
pub fn load_config() -> Result<Exclusions, Box<dyn std::error::Error>> {
    let config_directory = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("indexed-filesystem-search");

    fs::create_dir_all(&config_directory)?;

    let config_path = config_directory.join("config.toml");

    if !config_path.exists() {
        let exclusions = default_exclusions();

        let toml = toml::to_string_pretty(&exclusions)?;
        fs::write(&config_path, toml)?;

        return Ok(exclusions);
    }

    let contents = fs::read_to_string(&config_path)?;
    let exclusions: Exclusions = toml::from_str(&contents)?;

    Ok(exclusions)
}

//function that returns default exclusions, some of the ones I do not need, this can be changed inside of the toml file
//it is only called when the config.toml file does not exist, so when it is created these are the exclusions which are assigned
fn default_exclusions() -> Exclusions {
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
