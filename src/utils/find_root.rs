use std::{
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use crate::utils::{fs, fs::DirEntry};

const ROOT_MARKERS: [&str; 9] = [
    ".git",
    "pom.xml",
    ".idea",
    "build.gradle",
    "build.gradle.kts",
    "lazy-java.toml",
    "lazy-java.lock",
    ".project",
    ".classpath",
];

pub fn find_root(start: &Path) -> Result<Option<PathBuf>, io::Error> {
    log::debug!("Looking for root in {}", start.to_string_lossy());
    let dirs = list_dir(start)?;

    log::debug!("Found {} entries in directory", dirs.len());
    for dir in dirs {
        if let Some(name) = dir.file_name().to_str()
            && ROOT_MARKERS.contains(&name)
        {
            log::debug!("Found root marker: {}", name);
            return Ok(Some(start.to_path_buf()));
        }
    }
    match start.parent() {
        Some(parent) => {
            log::debug!(
                "No root marker found in {}, searching parent: {}",
                start.to_string_lossy(),
                parent.to_string_lossy()
            );
            Ok(find_root(parent)?)
        }
        None => {
            log::debug!("Reached filesystem root: {}", start.to_string_lossy());
            Ok(None)
        }
    }
}
pub fn find_file_in_dir(dir: &Path, search_name: &str) -> Result<DirEntry, io::Error> {
    log::debug!("Searching for file '{}' in {:?}", search_name, dir);
    for file in list_dir(dir)? {
        if let Some(name) = file.file_name().to_str()
            && name == search_name
        {
            log::debug!("Found file: {}", search_name);
            return Ok(file);
        }
    }

    log::warn!("File not found: {} in {:?}", search_name, dir);
    Err(io::Error::new(ErrorKind::NotFound, "Couldnt find file"))
}

pub fn list_dir(path: &Path) -> Result<Vec<DirEntry>, io::Error> {
    log::debug!("Opening directory: {:?}", path);
    let dir = fs::read_dir(path).map_err(|e| {
        log::error!("Error opening directory {:?}: {}", path, e);
        e
    })?;
    dir.collect()
}

#[cfg(test)]
mod tests {
    use std::{env, io};

    use crate::utils::find_root::{find_file_in_dir, find_root};

    #[test]
    fn find_file_test() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_file_test");

        let _file = find_file_in_dir(&current, "file1.txt")?;
        let _file1 = find_file_in_dir(&current, "file2.txt")?;

        let file3 = find_file_in_dir(&current, "not here");

        assert!(
            file3.is_err(),
            "When finding a file that doesnt exist expected error"
        );

        return Ok(());
    }

    #[test]
    fn find_root_test() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_root_test");

        let root = current.clone();
        current.push("dir1");
        let one_level = current.clone();
        current.push("dir2");
        let two_level = current.clone();

        println!("Finding Root at {:?}", root);
        let find1 = find_root(&root);
        println!("Finding Root at {:?}", one_level);
        let find2 = find_root(&one_level);
        println!("Finding Root at {:?}", two_level);
        let find3 = find_root(&two_level);

        assert!(find1.is_ok(), "Could not find root at the root level");
        assert!(find2.is_ok(), "Could not find root one level down");
        assert!(find3.is_ok(), "Could not find root two levels down");

        return Ok(());
    }
}
