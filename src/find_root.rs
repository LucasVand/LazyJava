use std::{
    fs::{self, DirEntry},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

const ROOT_MARKERS: [&'static str; 5] = [
    ".git",
    "pom.xml",
    ".idea",
    "build.gradle",
    "build.gradle.kts",
];

pub fn find_root(start: &Path) -> Result<PathBuf, io::Error> {
    let dirs = list_dir(start)?;

    for dir in dirs {
        if let Some(name) = dir.file_name().to_str() {
            if ROOT_MARKERS.contains(&name) {
                return Ok(start.to_path_buf());
            }
        }
    }
    return match start.parent() {
        Some(parent) => Ok(find_root(parent)?),
        None => Err(io::Error::new(ErrorKind::NotFound, "No root marker found")),
    };
}
pub fn find_file_in_dir(dir: &Path, search_name: &str) -> Result<DirEntry, io::Error> {
    for file in list_dir(dir)? {
        if let Some(name) = file.file_name().to_str() {
            if name == search_name {
                return Ok(file);
            }
        }
    }

    return Err(io::Error::new(ErrorKind::NotFound, "Couldnt find file"));
}

pub fn list_dir(path: &Path) -> Result<Vec<DirEntry>, io::Error> {
    let dir = fs::read_dir(path)?;
    return dir.collect();
}
