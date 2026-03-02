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

        let find1 = find_root(&root);
        let find2 = find_root(&one_level);
        let find3 = find_root(&two_level);

        assert!(find1.is_ok(), "Could not find root at the root level");
        assert!(find2.is_ok(), "Could not find root one level down");
        assert!(find3.is_ok(), "Could not find root two levels down");

        return Ok(());
    }
}
