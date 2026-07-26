use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use colored::Colorize;

use crate::{
    args::ImportPomArgs,
    config::{Config, ConfigDependancy, ConfigProject},
    maven_central::{MavenIdBuf, PartialMavenIdBuf, pom::MavenPom},
};

use super::ImportError;

fn dep_mgmt_map(pom: &MavenPom) -> HashMap<u64, String> {
    let mut map = HashMap::new();
    if let Some(mgmt) = &pom.dependency_management {
        for dep in &mgmt.dependencies.dependency {
            if let Some(version) = &dep.version {
                let mut hasher = DefaultHasher::new();
                (dep.group_id.as_str(), dep.artifact_id.as_str()).hash(&mut hasher);
                map.insert(hasher.finish(), version.clone());
            }
        }
    }
    map
}

pub fn import_pom(root: &Path, args: &ImportPomArgs) -> Result<(), ImportError> {
    let toml_path = root.join("lazy-java.toml");
    if toml_path.exists() && !args.overwrite {
        eprintln!(
            "{} lazy-java.toml already exists. Use --overwrite to replace it.",
            "Skipping:".yellow().bold()
        );
        return Ok(());
    }

    let content = fs::read_to_string(&root.join(&args.pom_path))?;
    let pom: MavenPom = quick_xml::de::from_str(&content)?;

    let mgmt_map = dep_mgmt_map(&pom);

    let mut dependancies: Vec<(PartialMavenIdBuf, ConfigDependancy)> = Vec::new();
    if let Some(deps) = &pom.dependencies {
        for dep in &deps.dependency {
            let version = match &dep.version {
                Some(v) => v.clone(),
                None => {
                    let mut hasher = DefaultHasher::new();
                    (dep.group_id.as_str(), dep.artifact_id.as_str()).hash(&mut hasher);
                    match mgmt_map.get(&hasher.finish()) {
                        Some(v) => v.clone(),
                        None => {
                            eprintln!(
                                "{} Skipping {}:{} — no version in dep or dependencyManagement",
                                "Warning:".yellow().bold(),
                                dep.group_id,
                                dep.artifact_id,
                            );
                            continue;
                        }
                    }
                }
            };
            let key = PartialMavenIdBuf::new(&dep.group_id, &dep.artifact_id);
            dependancies.push((
                key,
                MavenIdBuf::new(&dep.group_id, &dep.artifact_id, version).into(),
            ));
        }
    }
    let dependancies: HashMap<_, _> = dependancies.into_iter().collect();

    let config = Config {
        project: ConfigProject {
            name: pom.artifact_id.clone(),
            group: Some(pom.group_id.clone()),
            artifact: Some(pom.artifact_id.clone()),
            version: Some(pom.version.clone()),
        },
        dependancies,
        ..Default::default()
    };

    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(root.join("lazy-java.toml"), toml_str)?;
    println!("{} lazy-java.toml from pom.xml", "Imported".green().bold());

    Ok(())
}
