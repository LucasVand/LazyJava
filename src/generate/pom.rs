use std::{collections::HashMap, path::PathBuf};

use colored::Colorize;
use quick_xml::se::Serializer;
use serde::Serialize;

use crate::{
    Context,
    generate::GenerateError,
    lock_file::LockFile,
    maven_central::pom::{
        AnnotationProcessorPaths, Build, Configuration, Dependencies, Dependency, DependencyType,
        MavenPom, Plugin, Plugins, ProcessorPath, Properties, Scope,
    },
    utils::{IOError, XmlSerializeError, fs, jdk_version::desired_jdk_version},
};

pub fn generate_pom(ctx: &Context) -> Result<(), GenerateError> {
    let pom = create_pom(ctx)?;
    let mut buffer = String::new();

    let mut serializer = Serializer::new(&mut buffer);
    serializer.indent(' ', 4);
    let pom_path = ctx.root.join("pom.xml");

    pom.serialize(serializer)
        .map_err(|source| XmlSerializeError::new("serializing pom.xml", &pom_path, source))?;

    fs::write(&pom_path, buffer)
        .map_err(|source| IOError::new("writing pom.xml", pom_path, source))?;
    println!("{} pom.xml", "Generated".green().bold());

    Ok(())
}

fn create_pom(ctx: &Context) -> Result<MavenPom, GenerateError> {
    let project = assert(ctx.config.project(), "project")?;
    let group_id = assert(project.group(), "group")?;
    let artifact_id = assert(project.artifact(), "artifact")?;
    let version_id = assert(project.version(), "version")?;

    let jdk = desired_jdk_version(None, Some(ctx));

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

    let mut dependencies: Vec<Dependency> = lockfile
        .packages
        .into_iter()
        .filter(|d| d.root)
        .map(|d| Dependency {
            group_id: d.id.group,
            artifact_id: d.id.artifact,
            version: Some(d.id.version),
            scope: d.scope,
            optional: false,
            dependency_type: d.packaging,
            classifier: None,
            system_path: None,
        })
        .chain(
            lockfile
                .local_packages
                .into_iter()
                .map(|local_d| {
                    let path = if local_d.path.is_relative() {
                        PathBuf::from("${project.basedir}").join(&local_d.path)
                    } else {
                        local_d.path.clone()
                    };
                    Dependency {
                        group_id: format!("local-{}-group", local_d.name),
                        artifact_id: format!("local-{}-artifact", local_d.name),
                        version: None,
                        scope: Scope::System,
                        optional: false,
                        dependency_type: local_d.packaging,
                        classifier: None,
                        system_path: Some(path.to_string_lossy().to_string()),
                    }
                }),
        )
        .collect();
    dependencies.sort_by(|a, b| {
        a.group_id
            .cmp(&b.group_id)
            .then(a.artifact_id.cmp(&b.artifact_id))
    });
    let dependencies = Dependencies {
        dependency: dependencies,
    };

    Ok(MavenPom {
        model_version: Some("4.0.0".into()),
        group_id: group_id.to_string(),
        artifact_id: artifact_id.to_string(),
        version: version_id.to_string(),
        packaging: DependencyType::Jar,
        dependencies: Some(dependencies),
        dependency_management: None,
        properties: Properties {
            map: HashMap::from([("maven.compiler.release".to_string(), jdk)]),
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
            value_name,
            generated_value: "pom.xml",
        }),
    }
}
