use std::{ffi::OsStr, fs, io, path::PathBuf, sync::LazyLock};

use regex::Regex;

#[derive(Debug)]
pub struct MainClass {
    pub path: PathBuf,
    pub classname: String,
}

static MAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return Regex::new(r"public static void main(.*) ").unwrap();
});
static CLASS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return Regex::new(r"(public[ ]*)?class (?<class>[^ ]*) ").unwrap();
});
pub fn find_main_classes(root: &PathBuf) -> Result<Vec<MainClass>, io::Error> {
    let mut main_classes: Vec<MainClass> = Vec::new();
    let java_files = find_java_files(root)?;
    for file in java_files {
        let content = fs::read_to_string(&file)?;
        let mains = MAIN_REGEX.find_iter(&content);
        let classes = CLASS_REGEX.captures(&content);
        if let Some(classes) = classes {
            let class_name = classes.name("class").unwrap();
            for _ in mains {
                main_classes.push(MainClass {
                    path: file.clone(),
                    classname: class_name.as_str().to_string(),
                });
            }
        }
    }
    return Ok(main_classes);
}

fn find_java_files(root: &PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let mut java_files: Vec<PathBuf> = Vec::new();

    for file in fs::read_dir(root)? {
        let f = file?.path();

        if f.is_dir() {
            let mut res = find_java_files(&f)?;
            java_files.append(&mut res);
        }

        if f.extension() == Some(OsStr::new("java")) {
            if f.is_file() {
                java_files.push(f);
            }
        }
    }
    return Ok(java_files);
}
