use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use regex::Captures;

use crate::{CLASS_REGEX, MAIN_REGEX, PACKAGE_REGEX};

#[derive(Debug)]
pub struct MainClass {
    pub path: PathBuf,
    pub classname: String,
    pub full_package_name: String,
}

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

    let mut removed_inner_content = content.to_string();
    let main = MAIN_REGEX.captures(content);

    let mut main_vec: Vec<MainClass> = Vec::new();
    if let Some(cap) = main {
        removed_inner_content.replace_range(cap.get_match().range(), "");
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

    if let Some(inner_class) = CLASS_REGEX.captures(&removed_inner_content) {
        let mut pack = package.to_string();
        pack.push_str(&format!(".{}", &classname));

        let mut classes = find_main_class(inner_class, &pack, file)?;
        main_vec.append(&mut classes);
    }

    Ok(main_vec)
}

pub fn find_java_files(root: &Path) -> Result<Vec<PathBuf>, io::Error> {
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

#[cfg(test)]
mod tests {
    use std::{env, fs, io, path::PathBuf};

    use crate::utils::find_main::{MainClass, find_main_classes};

    #[test]
    fn find_main_test() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_main_classes_test");

        let classes = find_main_classes(&current)?;

        let expect1 = MainClass {
            path: PathBuf::from("./test_filesystem/find_main_classes_test/Test1.java"),
            classname: "Test1".to_string(),
            full_package_name: "Test1".to_string(),
        };
        let expect2 = MainClass {
            path: PathBuf::from("./test_filesystem/find_main_classes_test/Test2.java"),
            classname: "Test2".to_string(),
            full_package_name: "Test2".to_string(),
        };
        let expect3 = MainClass {
            path: PathBuf::from("./test_filesystem/find_main_classes_test/Test3.java"),
            classname: "Test3".to_string(),
            full_package_name: "Test3".to_string(),
        };
        let expect4 = MainClass {
            path: PathBuf::from("./test_filesystem/find_main_classes_test/Test4.java"),
            classname: "Test4".to_string(),
            full_package_name: "Test4".to_string(),
        };
        let expect5 = MainClass {
            path: PathBuf::from("./test_filesystem/find_main_classes_test/dir1/Test5.java"),
            classname: "Test5".to_string(),
            full_package_name: "dir1.Test5".to_string(),
        };

        let expected = vec![expect3, expect2, expect4, expect1, expect5];

        dbg!(&classes);
        for index in 0..5 {
            let main = &classes[index];
            let expect = expected.iter().find(|m| {
                return m.full_package_name == main.full_package_name;
            });

            assert!(
                expect.is_some(),
                "Couldnt Find Expected Main Class Matching a Class Found "
            );
            let expect = expect.unwrap();
            let con_main = fs::canonicalize(&main.path)?;
            let con_expec = fs::canonicalize(&expect.path)?;

            assert_eq!(con_main, con_expec, "Paths did not match");
            assert_eq!(main.classname, expect.classname, "Classnames did not match");
            assert_eq!(
                main.full_package_name, expect.full_package_name,
                "Package names did not match"
            );
            println!("Passed Test {}", index);
        }

        return Ok(());
    }
}
