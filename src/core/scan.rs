use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn scan_pages(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut dirs = vec![dir.as_ref().to_path_buf()];
    let mut files = Vec::new();

    while let Some(current) = dirs.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                dirs.push(entry.path());
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            if path.extension() == Some(OsStr::new("rs")) {
                files.push(path);
            }
        }
    }

    Ok(files)
}
