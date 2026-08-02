use std::{
    collections::HashSet,
    ffi::OsStr,
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
        classpath::{Attribute, Attributes, Classpath, ClasspathEntry},
        classpath_error::ClasspathError,
    },
    utils::fs,
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
    fn defualt_entries(ctx: &Context) -> Vec<ClasspathEntry> {
        let mut entries = Vec::new();
        entries.push(ClasspathEntry {
            kind: "src".into(),
            path: ctx.relative_src.clone(),
            including: Some("**/*.java".to_string()),
            output: None,
            attributes: None,
        });
        entries.push(ClasspathEntry {
            kind: "src".into(),
            path: format!("{}/{}", ctx.relative_target, ctx.relative_src_generated),
            including: Some("**/*.java".to_string()),
            output: None,
            attributes: Some(Attributes {
                list: vec![
                    Attribute {
                        name: "optional".to_string(),
                        value: "true".into(),
                    },
                    Attribute {
                        name: "ignore_optional_problems".into(),
                        value: "true".into(),
                    },
                ],
            }),
        });
        entries.push(ClasspathEntry {
            kind: "output".into(),
            path: format!("{}/{}", ctx.relative_target, ctx.relative_bin),
            including: None,
            output: None,
            attributes: None,
        });

        entries.push(ClasspathEntry {
            kind: "con".into(),
            path: JAVA_CONTAINER.into(),
            including: None,
            output: None,
            attributes: None,
        });
        entries
    }

    fn create(ctx: &Context) -> Result<Classpath, ClasspathError> {
        log::debug!("Creating classpath from project structure");

        let dir = Self::lib_files(ctx)?;
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

        entries.extend(Self::defualt_entries(ctx));

        log::debug!("Created classpath with {} total entries", entries.len());
        let classpath = Classpath { entries };

        Ok(classpath)
    }
    fn lib_files(ctx: &Context) -> Result<Vec<PathBuf>, ClasspathError> {
        log::debug!("Scanning library directory: {:?}", &ctx.lib_annotations);
        let mut java_files: Vec<PathBuf> = Vec::new();

        java_files.extend(Self::find_jars(&ctx.lib)?);
        java_files.extend(Self::find_jars(&ctx.lib_annotations)?);

        log::debug!("Found {} JAR files in library directory", java_files.len());
        Ok(java_files)
    }
    fn find_jars(root: &Path) -> Result<Vec<PathBuf>, ClasspathError> {
        let mut java_files: Vec<PathBuf> = Vec::new();
        for file in walkdir::WalkDir::new(root).into_iter().flatten() {
            let path = file.path();
            if path.extension() == Some(OsStr::new("jar")) {
                java_files.push(path.to_path_buf());
            }
        }
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

        let libs: Vec<String> = Self::lib_files(ctx)?
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        let equal = libs == classpath_libs;

        if !equal {
            log::warn!("Classpath library entries do not match filesystem");
            return Ok(false);
        } else {
            log::debug!("Classpath is valid");
        }

        let expected_defaults: HashSet<ClasspathEntry> =
            Self::defualt_entries(ctx).into_iter().collect();

        let defaults: HashSet<&ClasspathEntry> = classpath
            .entries
            .iter()
            .filter(|e| e.kind != "lib")
            .collect();

        for expected in expected_defaults {
            if !defaults.contains(&expected) {
                return Ok(false);
            }
        }
        Ok(true)
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
