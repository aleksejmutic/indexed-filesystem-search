use std::env;
use walkdir::WalkDir;

pub fn scan() -> Result<(), walkdir::Error> {
    let home_directory = match env::home_dir() {
        Some(path) => path,
        None => panic!("Could not determine the home directory."),
    };

    for entry in WalkDir::new(home_directory) {
        println!("{}", entry?.path().display());
    }

    Ok(())
}
