use std::ffi::OsStr;
use std::path;
use std::{ffi::OsString, path::Path};

use log::{debug, warn};
use walkdir::WalkDir;

pub fn join_directory(path: &Path, seperator: char) -> String {
    let mut des = String::new();

    let mut first = true;
    for entry in WalkDir::new(path).into_iter() {
        if let Ok(f) = entry
            && f.file_type().is_file()
        {
            if let Ok(ab) = path::absolute(f.path()) {
                if let Some(valid_str) = ab.to_str() {
                    if !first {
                        des.push(seperator);
                    }
                    des.push_str(valid_str);
                    first = false
                } else {
                    warn!("Directory is not valid UTF-8 skipping, {}", ab.display())
                }
            }
        }
    }
    debug!("Des Result: {:#?}", des);
    return des;
}

pub fn find_all_java_files(path: &Path) -> String {
    let mut des = String::new();

    for entry in WalkDir::new(path).into_iter() {
        if let Ok(f) = entry
            && f.file_type().is_file()
        {
            if let Ok(ab) = path::absolute(f.path())
                && ab.extension() == Some(OsStr::new("java"))
            {
                des.push_str(&ab.to_string_lossy());
                des.push(' ');
            }
        }
    }
    debug!("Find All Result: {:#?}", des);
    return des;
}
