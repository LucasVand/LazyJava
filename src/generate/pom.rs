use std::{collections::HashMap, fs};

use colored::Colorize;
use quick_xml::se::Serializer;
use serde::Serialize;

use crate::{
    Context,
    generate::GenerateError,
    lock_file::LockFile,
    maven_central::pom::{
        AnnotationProcessorPaths, Build, Configuration, DependancyType, Dependencies, Dependency,
        MavenPom, Plugin, Plugins, ProcessorPath, Properties, Scope,
    },
};

pub fn generate_pom(ctx: &Context) -> Result<(), GenerateError> {
    let pom = create_pom(ctx)?;
    let mut buffer = String::new();

    let mut serializer = Serializer::new(&mut buffer);
    serializer.indent(' ', 4);

    pom.serialize(serializer)?;

    if !ctx.dry_run {
        fs::write(ctx.root.join("pom.xml"), buffer)?;
    }
    println!("{} pom.xml", "Generated".green().bold());

    return Ok(());
}

fn create_pom(ctx: &Context) -> Result<MavenPom, GenerateError> {
    let project = assert(ctx.config.project(), "project")?;
    let group_id = assert(project.group(), "group")?;
    let artifact_id = assert(project.artifact(), "artifact")?;
    let version_id = assert(project.version(), "version")?;

    if ctx.config.processors().is_some_and(|p| !p.is_empty()) {
        eprintln!(
            "{} Local annotation processors are not supported in generated pom.xml. Only Maven-sourced processors are included.",
            "Warning:".yellow().bold()
        );
    }

    let lockfile = LockFile::fetch(&ctx.root)?;

    let build = {
        let processor_deps: Vec<_> = lockfile
            .packages
            .iter()
            .filter(|p| !p.annotations.is_empty())
            .collect();
        if !processor_deps.is_empty() {
            Some(Build {
                plugins: Some(Plugins {
                    plugin: vec![Plugin {
                        group_id: Some("org.apache.maven.plugins".into()),
                        artifact_id: Some("maven-compiler-plugin".into()),
                        version: Some("3.11.0".into()),
                        configuration: Some(Configuration {
                            annotation_processor_paths: Some(AnnotationProcessorPaths {
                                path: processor_deps
                                    .into_iter()
                                    .map(|p| ProcessorPath {
                                        group_id: p.id.group.clone(),
                                        artifact_id: p.id.artifact.clone(),
                                        version: p.id.version.clone(),
                                    })
                                    .collect(),
                            }),
                        }),
                    }],
                }),
                source: None,
                target: None,
            })
        } else {
            None
        }
    };

    let mut dependancies: Vec<Dependency> = lockfile
        .packages
        .into_iter()
        .filter(|d| d.root)
        .map(|d| Dependency {
            group_id: d.id.group,
            artifact_id: d.id.artifact,
            version: Some(d.id.version),
            scope: Scope::Compile,
            optional: false,
            dependency_type: d.packaging,
            classifier: None,
        })
        .collect();
    dependancies.sort_by(|a, b| {
        a.group_id
            .cmp(&b.group_id)
            .then(a.artifact_id.cmp(&b.artifact_id))
    });
    let dependancies = Dependencies {
        dependency: dependancies,
    };

    Ok(MavenPom {
        model_version: Some("4.0.0".into()),
        group_id: group_id.to_string(),
        artifact_id: artifact_id.to_string(),
        version: version_id.to_string(),
        packaging: DependancyType::Jar,
        dependencies: Some(dependancies),
        dependency_management: None,
        properties: Properties {
            map: HashMap::new(),
        },
        parent: None,
        build,
        dependency_management_map: HashMap::new(),
    })
}
fn assert<T>(value: Option<T>, value_name: &'static str) -> Result<T, GenerateError> {
    match value {
        Some(v) => Ok(v),
        None => Err(GenerateError::MissingValue {
            value_name: value_name,
            generated_value: "pom.xml",
        }),
    }
}
