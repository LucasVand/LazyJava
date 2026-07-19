use std::ffi::OsStr;
use std::path;
use std::{ffi::OsString, path::Path};

use log::debug;
use walkdir::WalkDir;

pub fn destructure_dir(path: &Path) -> OsString {
    let mut des = OsString::new();
    let seperator: &str = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    let mut first = true;
    for entry in WalkDir::new(path).into_iter() {
        if let Ok(f) = entry
            && f.file_type().is_file()
        {
            if let Ok(ab) = path::absolute(f.path()) {
                if !first {
                    des.push(seperator);
                }
                des.push(ab.as_os_str());
                first = false
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
