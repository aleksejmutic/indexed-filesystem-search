# Indexed Filesystem Search

A Linux file launcher that searches indexed filesystem entries. Written in Rust and stores file entries in SQLite. The Pop!_OS one does not do any sort of find.
It scans the user's home directory, or any set default location, while supporting configurable file, extension, and directory exclusions through a TOML configuration file.

The project is currently focused on building the filesystem index and keeping it synchronized with the actual filesystem.

It will eventually have some CLI and a UI so when a user presses the Super button they can run the app. The UI part will probably be a fork of this, since I want to separate the tool from the whole app launcher.

find, locate, plocate, fd, recoll are all tools that do this, but I was bored and wanted to create something myself.

Important: Most of the issues created are to be closed, I just like to keep them opened as they serve me as some form of documentation of where I am headed.
