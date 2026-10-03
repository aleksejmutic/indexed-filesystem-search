# Indexed Filesystem Search

A Linux filesystem search application written in Rust that indexes file metadata in SQLite for quick searching.
It scans the user's home directory, or any set default location, while supporting configurable file, extension, and directory exclusions through a TOML configuration file.
The project is currently focused on building the filesystem index and keeping it synchronized with the actual filesystem.
