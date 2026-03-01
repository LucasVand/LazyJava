use std::{ffi::OsStr, fs, io, path::PathBuf, sync::LazyLock};

use regex::{Captures, Regex, RegexBuilder};

#[derive(Debug)]
pub struct MainClass {
    pub path: PathBuf,
    pub classname: String,
    pub full_package_name: String,
}
static PACKAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return RegexBuilder::new(r"^\s*package\s*(?<package>.*);")
        .unicode(true)
        .build()
        .unwrap();
});

static MAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return RegexBuilder::new(r"public static void main(.*) \{(?<content>[\s\S]*)\}")
        .unicode(true)
        .multi_line(true)
        .build()
        .unwrap();
});
static CLASS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let re = RegexBuilder::new(
        r#"^\s*(?:(?:public|static|abstract|final|private)\s+)*class\s+(?<class>\S*)\s+(?:extend.*)*\s*(?:implements.*)*\s*\{(?<content>[\s\S]*)\}"#,
    ).multi_line(true).unicode(true).build();
    return re.unwrap();
});
pub fn find_main_classes(root: &PathBuf) -> Result<Vec<MainClass>, io::Error> {
    let mut main_classes: Vec<MainClass> = Vec::new();
    let java_files = find_java_files(root)?;
    for file in java_files {
        let content = fs::read_to_string(&file)?;
        let class = CLASS_REGEX.captures(&content);
        let package_captures = PACKAGE_REGEX.captures(&content);

        let package = match package_captures {
            Some(cap) => {
                let package = cap.name("package").unwrap();
                package.as_str()
            }
            None => "",
        };

        if let Some(class) = class {
            let mut found_classes = find_main_class(class, package, &file)?;

            main_classes.append(&mut found_classes);
        }
    }
    return Ok(main_classes);
}
fn find_main_class(
    class: Captures<'_>,
    package: &str,
    file: &PathBuf,
) -> Result<Vec<MainClass>, io::Error> {
    let classname = class.name("class").unwrap().as_str();
    let content = class.name("content").unwrap().as_str();

    let main = MAIN_REGEX.captures(content);

    let mut main_vec: Vec<MainClass> = Vec::new();
    if let Some(_main) = main {
        let full_package = if package != "" {
            format!("{}.{}", package, classname)
        } else {
            classname.to_string()
        };

        let class = MainClass {
            path: file.to_path_buf().clone(),
            classname: classname.to_string(),
            full_package_name: full_package,
        };

        main_vec.push(class);
    }

    if let Some(inner_class) = CLASS_REGEX.captures(content) {
        let mut pack = package.to_string();
        pack.push_str(&format!(".{}", &classname));

        let mut classes = find_main_class(inner_class, &pack, file)?;
        main_vec.append(&mut classes);
    }

    Ok(main_vec)
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
