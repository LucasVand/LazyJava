use std::{
    ffi::OsStr,
    fs::{self},
    io::{self, Write},
    path::{self, Path, PathBuf},
};

use quick_xml::events::{BytesDecl, Event};
use quick_xml::{SeError, Writer};
use serde::Serialize;
use std::io::Cursor;

use crate::{
    Context,
    lsp::{
        classpath::{Classpath, ClasspathEntry},
        classpath_error::ClasspathError,
    },
};

const JAVA_CONTAINER: &str = "org.eclipse.jdt.launching.JRE_CONTAINER";

impl Classpath {
    fn parse(path: &Path) -> Result<Self, ClasspathError> {
        log::debug!("Parsing classpath file: {:?}", path);
        let file = fs::read_to_string(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => {
                log::warn!("Classpath file not found: {:?}", path);
                ClasspathError::NoClasspathFile
            }
            _ => {
                log::error!("Error reading classpath file {:?}: {}", path, e);
                ClasspathError::OSErrorClasspath(e)
            }
        })?;

        let classpath: Classpath = quick_xml::de::from_str(&file)?;
        log::debug!(
            "Successfully parsed classpath with {} entries",
            classpath.entries.len()
        );

        Ok(classpath)
    }
    pub fn generate(ctx: &Context) -> Result<(), ClasspathError> {
        log::info!("Generating classpath file");
        let classpath = Self::create(ctx)?;

        let serialized = Self::to_pretty_xml(&classpath)?;

        let path = ctx.root.join(".classpath");

        log::debug!("Writing classpath to {:?}", path);
        fs::write(&path, serialized).map_err(|e| {
            log::error!("Failed to write classpath file: {}", e);
            ClasspathError::ClasspathWrite(
                path::absolute(path).unwrap().to_string_lossy().into(),
                e,
            )
        })?;

        log::info!("Classpath file generated successfully");
        Ok(())
    }

    fn create(ctx: &Context) -> Result<Classpath, ClasspathError> {
        log::debug!("Creating classpath from project structure");

        let dir = Self::lib_files(&ctx.lib)?;
        log::debug!("Found {} library files", dir.len());

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
            path: ctx.relative_src.clone(),
            including: None,
            output: Some(ctx.relative_bin.clone()),
            attributes: None,
        });

        entries.push(ClasspathEntry {
            kind: "con".into(),
            path: JAVA_CONTAINER.into(),
            including: None,
            output: None,
            attributes: None,
        });

        log::debug!("Created classpath with {} total entries", entries.len());
        let classpath = Classpath { entries };

        Ok(classpath)
    }
    fn lib_files(lib: &Path) -> Result<Vec<PathBuf>, ClasspathError> {
        log::debug!("Scanning library directory: {:?}", lib);
        let mut java_files: Vec<PathBuf> = Vec::new();

        let files = fs::read_dir(lib).map_err(|e| {
            log::error!("Failed to read library directory {:?}: {}", lib, e);
            ClasspathError::OSErrorLib(path::absolute(lib).unwrap().to_string_lossy().into(), e)
        })?;

        for file in files {
            let f = file
                .map_err(|e| {
                    log::error!("Error reading file in library directory: {}", e);
                    ClasspathError::OSErrorLib(
                        path::absolute(lib).unwrap().to_string_lossy().into(),
                        e,
                    )
                })?
                .path();

            if f.is_dir() {
                let mut res = Self::lib_files(&f)?;
                java_files.append(&mut res);
            }

            if f.extension() == Some(OsStr::new("jar")) && f.is_file() {
                log::debug!("Found JAR file: {:?}", f);
                java_files.push(f);
            }
        }
        log::debug!("Found {} JAR files in library directory", java_files.len());
        Ok(java_files)
    }
    fn validate(ctx: &Context) -> Result<bool, ClasspathError> {
        log::debug!("Validating classpath file");
        let root = &ctx.root;

        let classpath = match Self::parse(&root.join(".classpath")) {
            Ok(cp) => cp,
            Err(ClasspathError::NoClasspathFile) => {
                log::debug!("Classpath file does not exist, needs generation");
                return Ok(false);
            }
            Err(e) => return Err(e),
        };

        let classpath_libs: Vec<String> = classpath
            .entries
            .iter()
            .filter(|entry| &entry.kind == "lib")
            .map(|entry| entry.path.clone())
            .collect();

        let classpath_src = classpath.entries.iter().find(|entry| entry.kind == "src");
        if let Some(classpath_src) = classpath_src {
            let c_src = path::absolute(Path::new(&classpath_src.path))
                .map_err(|_| ClasspathError::PathError(classpath_src.path.to_string()))?;
            let src = path::absolute(&ctx.src)
                .map_err(|_| ClasspathError::PathError(ctx.src.to_string_lossy().to_string()))?;

            let c_output =
                path::absolute(Path::new(&classpath_src.output.clone().unwrap_or_default()))
                    .map_err(|_| {
                        ClasspathError::PathError(classpath_src.output.clone().unwrap_or_default())
                    })?;
            let build = path::absolute(Path::new(&ctx.bin))
                .map_err(|_| ClasspathError::PathError(ctx.bin.to_string_lossy().to_string()))?;

            if (c_src != src) || (c_output != build) {
                log::debug!("Classpath source entry is out of date");
                log::debug!(
                    "Source equality: {}, Output eqaulity: {}",
                    c_src == src,
                    c_output == build
                );
                return Ok(false);
            }
        } else {
            log::debug!("No source entry found in classpath");
            return Ok(false);
        }

        let classpath_container = classpath.entries.iter().find(|e| e.kind == "con");
        if classpath_container.is_none() {
            return Ok(false);
        }

        let libs: Vec<String> = Self::lib_files(&ctx.lib)?
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        let equal = libs == classpath_libs;

        if !equal {
            log::warn!("Classpath library entries do not match filesystem");
        } else {
            log::debug!("Classpath is valid");
        }

        Ok(equal)
    }
    pub fn generate_if_stale(ctx: &Context) -> Result<(), ClasspathError> {
        log::debug!("Checking if classpath needs regeneration");
        if !(Self::validate(ctx)?) {
            log::info!("Classpath is stale, regenerating");
            Self::generate(ctx)?
        } else {
            log::debug!("Classpath is up to date");
        }
        Ok(())
    }
    fn to_pretty_xml<T: Serialize>(value: &T) -> Result<String, SeError> {
        let mut buffer = Cursor::new(Vec::new());

        // 4-space indentation
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 4);

        // Optional XML declaration
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .unwrap();

        // newline after declaration
        writer.get_mut().write_all(b"\n").unwrap();

        // serialize the actual XML
        writer.write_serializable("classpath", value)?;

        Ok(String::from_utf8(buffer.into_inner()).unwrap())
    }
}
