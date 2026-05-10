use std::{
    ffi::OsStr,
    fs::{self},
    io::{self},
    path::{self, Path, PathBuf},
};

use crate::{
    lazy_java::LazyJava,
    lsp::{
        classpath::{Classpath, ClasspathEntry},
        classpath_error::ClasspathError,
    },
};

impl Classpath {
    pub fn parse(path: &Path) -> Result<Self, ClasspathError> {
        let file = fs::read_to_string(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => ClasspathError::NoClasspathFile,
            _ => ClasspathError::OSErrorClasspath(e),
        })?;

        let classpath: Classpath = quick_xml::de::from_str(&file)?;

        Ok(classpath)
    }
    pub fn generate(lj: &LazyJava) -> Result<(), ClasspathError> {
        let classpath = Self::create(lj)?;

        let prefix = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
        let mut serialized = quick_xml::se::to_string(&classpath)?;
        serialized.insert_str(0, prefix);

        let mut path = lj.root.clone();
        path.push(".classpath");

        fs::write(&path, serialized).map_err(|e| {
            ClasspathError::ClasspathWrite(
                path::absolute(path).unwrap().to_string_lossy().into(),
                e,
            )
        })?;

        Ok(())
    }

    pub fn create(lj: &LazyJava) -> Result<Classpath, ClasspathError> {
        let src = &lj.args.global_args.source;
        let build = &lj.args.global_args.build;

        let dir = Self::lib_files(&lj.lib)?;

        let mut entries: Vec<ClasspathEntry> = dir
            .into_iter()
            .map(|entry| ClasspathEntry {
                kind: "lib".into(),
                path: entry.to_string_lossy().into(),
                including: None,
                output: None,
                attributes: None,
            })
            .collect();

        entries.push(ClasspathEntry {
            kind: "src".into(),
            path: src.into(),
            including: None,
            output: Some(build.into()),
            attributes: None,
        });

        let classpath = Classpath { entries };

        Ok(classpath)
    }
    fn lib_files(root: &Path) -> Result<Vec<PathBuf>, ClasspathError> {
        let mut java_files: Vec<PathBuf> = Vec::new();

        let files = fs::read_dir(root).map_err(|e| {
            ClasspathError::OSErrorLib(path::absolute(root).unwrap().to_string_lossy().into(), e)
        })?;

        for file in files {
            let f = file
                .map_err(|e| {
                    ClasspathError::OSErrorLib(
                        path::absolute(root).unwrap().to_string_lossy().into(),
                        e,
                    )
                })?
                .path();

            if f.is_dir() {
                let mut res = Self::lib_files(&f)?;
                java_files.append(&mut res);
            }

            if f.extension() == Some(OsStr::new("jar")) {
                if f.is_file() {
                    java_files.push(f);
                }
            }
        }
        return Ok(java_files);
    }
    pub fn validate(lj: &LazyJava) -> Result<bool, ClasspathError> {
        let root = &lj.root;

        let classpath = Self::parse(&root.join(".classpath"))?;

        let classpath_libs: Vec<String> = classpath
            .entries
            .iter()
            .filter(|entry| &entry.kind == "lib")
            .map(|entry| entry.path.clone())
            .collect();

        let classpath_src = classpath.entries.iter().find(|entry| entry.kind == "src");
        if let Some(classpath_src) = classpath_src {
            if !(classpath_src.path == lj.src.to_string_lossy())
                || (classpath_src.output == lj.build.to_str().map(|b| b.to_string()))
            {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        let libs: Vec<String> = Self::lib_files(root)?
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        let equal = libs == classpath_libs;

        return Ok(equal);
    }
    pub fn generate_if_stale(lj: &LazyJava) -> Result<(), ClasspathError> {
        if !(Self::validate(lj)?) {
            Self::generate(lj)?
        }
        return Ok(());
    }
}
