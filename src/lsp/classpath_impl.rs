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
    pub fn write_classpath(lj: &LazyJava) -> Result<(), ClasspathError> {
        let classpath = Self::generate(lj)?;

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

    pub fn generate(lj: &LazyJava) -> Result<Classpath, ClasspathError> {
        let src = &lj.args.global_args.source;
        let build = &lj.args.global_args.build;

        let dir = Self::lib_files(&lj.lib).map_err(|e| {
            ClasspathError::OSErrorLib(path::absolute(&lj.lib).unwrap().to_string_lossy().into(), e)
        })?;

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

        //             .map(|entry| {
        //                 let abs = path::absolute(entry).unwrap();
        //                 let abs_str = abs.to_string_lossy();
        //                 format!(r#"<classpathentry kind="lib" path="{abs_str}"/>"#)
        //             })
        //             .collect();
        //
        //         let entries_str = entries.join("\n");
        //         let classpath = format!(
        //             r#"
        // <?xml version="1.0" encoding="UTF-8"?>
        // <classpath>
        //   <!-- Source code -->
        //   <classpathentry including="**/*.java" kind="src" output="{build}" path="{src}"/>
        //   <!-- Libraries -->
        //   {entries_str}
        //
        //   <!-- Output directory -->
        //   <classpathentry kind="output" path="{build}"/>
        // </classpath>
        //     "#
        //         );

        Ok(classpath)
    }
    fn lib_files(root: &Path) -> Result<Vec<PathBuf>, io::Error> {
        let mut java_files: Vec<PathBuf> = Vec::new();

        for file in fs::read_dir(root)? {
            let f = file?.path();

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
}
