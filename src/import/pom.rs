use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use colored::Colorize;

use crate::{
    args::ImportPomArgs,
    config::ConfigTomlEdit,
    maven_central::{fetch_artifact_metadata, pom::MavenPom},
    utils::{IOError, XmlDeserializeError, fs},
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

    let pom_path = &root.join(&args.pom_path);
    let content =
        fs::read_to_string(pom_path).map_err(|e| IOError::new("reading pom.xml", pom_path, e))?;
    let pom: MavenPom = quick_xml::de::from_str(&content)
        .map_err(|e| XmlDeserializeError::new("parsing pom.xml", pom_path, e))?;

    let mgmt_map = dep_mgmt_map(&pom);

    let mut config = ConfigTomlEdit::parse("")?;
    if let Some(deps) = pom.dependencies {
        let mut tomldeps = config.dependancies_mut().get_or_insert(HashMap::new());
        for dep in deps.dependency {
            let version = match &dep.version {
                Some(v) => v.clone(),
                None => {
                    let mut hasher = DefaultHasher::new();
                    (dep.group_id.as_str(), dep.artifact_id.as_str()).hash(&mut hasher);
                    match mgmt_map.get(&hasher.finish()) {
                        Some(v) => v.clone(),
                        None => {
                            if let Ok(meta) =
                                fetch_artifact_metadata(&dep.group_id, &dep.artifact_id)
                                && let Some(latest) = meta.get_latest_version()
                            {
                                latest.to_string()
                            } else {
                                eprintln!(
                                    "{} Skipping {}:{} — no version in dep or dependencyManagement and cannot resolve maven artifact",
                                    "Warning:".yellow().bold(),
                                    dep.group_id,
                                    dep.artifact_id,
                                );
                                continue;
                            }
                        }
                    }
                }
            };
            let mut entry = tomldeps.insert_empty(&dep.artifact_id);
            entry.group_mut().set(dep.group_id.clone());
            entry.version_mut().set(version);
            entry.scope_mut().set(dep.scope);
        }
    }

    if !args.only_dependencies {
        {
            let mut p = config.project_mut().get_or_insert_empty();
            p.name_mut().set(pom.artifact_id.clone());
            p.group_mut().set(pom.group_id.clone());
            p.artifact_mut().set(pom.artifact_id.clone());
            p.version_mut().set(pom.version.clone());
        }

        if let Some(version) = pom.properties.map.get("java.version") {
            let mut v = config.setup_mut().get_or_insert_empty();
            v.release_mut().replace(version.to_string());
        }
        config
            .setup_mut()
            .get_or_insert_empty()
            .src_mut()
            .replace("src/main/java".to_string());

        config
            .resources_mut()
            .get_or_insert_empty()
            .external_mut()
            .replace(vec!["src/main/resources/**".to_string()]);
    }

    let toml_str = config.to_toml_string();

    fs::write(&toml_path, toml_str)
        .map_err(|e| IOError::new("writing lazy-java.toml", toml_path, e))?;
    println!("{} lazy-java.toml from pom.xml", "Imported".green().bold());

    Ok(())
}
