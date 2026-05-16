use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

use log::{debug, warn};
use regex::Regex;

use crate::maven_central::{
    MavenError, get_pom,
    pom::pom::{MavenPom, Scope},
};

pub struct MavenPomTree {
    pub root: (String, String, String),
    pub poms: HashMap<u64, MavenPom>,
}

impl MavenPomTree {
    pub fn new(group: String, artifact: String, version: String) -> Result<Self, MavenError> {
        log::info!("Creating POM tree for {}:{}:{}", group, artifact, version);

        let mut hash_map = HashMap::new();
        Self::resolve_related_poms(&group, &artifact, &version, &mut hash_map)?;

        log::info!("POM tree created with {} total POMs", hash_map.len());
        Ok(MavenPomTree {
            root: (group, artifact, version),
            poms: hash_map,
        })
    }
    fn resolve_related_poms<'a>(
        group: &str,
        artifact: &str,
        version: &str,
        map: &'a mut HashMap<u64, MavenPom>,
    ) -> Result<&'a MavenPom, MavenError> {
        // TODO: every time i get a pom i need to use the current map to resolve the properties
        // then i neeed to fetch the related poms and do another resolve
        //
        // ISSUE: sometimes the parent propeties are not being added to the childs this might be becuase
        // it does not work for more then a single level idk??
        log::debug!(
            "Resolving related POMs for {}:{}:{}",
            group,
            artifact,
            version
        );
        let hash = Self::hash_maven_id(group, artifact, version);

        let pom: Result<MavenPom, MavenError> = match map.remove(&hash) {
            Some(pom) => {
                log::debug!(
                    "Cache hit POM with hash: {} -> {}:{}:{}",
                    hash,
                    group,
                    artifact,
                    version
                );
                Ok(pom)
            }
            None => {
                let pom = get_pom(group, artifact, version)?;
                log::debug!(
                    "Fetched POM with hash: {} -> {}:{}:{}",
                    hash,
                    group,
                    artifact,
                    version
                );

                Ok(pom)
            }
        };

        let mut pom = pom?;

        Self::resolve_properties_inital(&mut pom);

        if let Some(parent) = &pom.parent {
            log::debug!(
                "Found parent POM: {}:{}:{}",
                parent.group_id,
                parent.artifact_id,
                parent.version
            );
            let parent_pom = Self::resolve_related_poms(
                &parent.group_id,
                &parent.artifact_id,
                &parent.version,
                map,
            )?;

            pom.properties.map.extend(parent_pom.properties.map.clone());
            pom.dependency_management_map
                .extend(parent_pom.dependency_management_map.clone());

            Self::resolve_properties_inital(&mut pom);
        }

        if let Some(dep_management) = &pom.dependency_management {
            for dep in &dep_management.dependencies.dependency {
                let scope = &dep.scope;
                if *scope != Scope::Import {
                    continue;
                }
                let version = dep.version.as_ref().expect("Bom is missing version");

                log::debug!(
                    "Found BOM import: {}:{}:{}",
                    dep.group_id,
                    dep.artifact_id,
                    version
                );
                let bom_pom =
                    Self::resolve_related_poms(&dep.group_id, &dep.artifact_id, version, map)?;

                pom.properties.map.extend(bom_pom.properties.map.clone());
                pom.dependency_management_map
                    .extend(bom_pom.dependency_management_map.clone());
            }
        }

        if let Some(deps) = &pom.dependencies {
            for dep in &deps.dependency {
                let scope = &dep.scope;
                if ![Scope::Compile, Scope::Runtime].contains(&scope) {
                    continue;
                }

                let version = dep.version.as_ref().unwrap_or_else(|| {
                    let dep_bom_hash = Self::hash_maven_bom_id(&dep.group_id, &dep.artifact_id);
                    let version = pom.dependency_management_map.get(&dep_bom_hash);

                    debug!(
                        "Found versioning in Bom for {}:{}, version: {}",
                        &dep.group_id,
                        &dep.artifact_id,
                        version.unwrap_or(&"(Blank)".to_string())
                    );
                    version.expect(&format!(
                        "Expected to find version in bom {:#?}",
                        pom.dependency_management_map
                    ))
                });

                log::debug!(
                    "Resolving transitive dependency: {}:{}:{}",
                    dep.group_id,
                    dep.artifact_id,
                    version
                );
                let dep_pom =
                    Self::resolve_related_poms(&dep.group_id, &dep.artifact_id, &version, map)?;

                pom.properties.map.extend(dep_pom.properties.map.clone());
            }
        }
        Self::resolve_properties_final(&mut pom);

        map.insert(hash, pom);

        Ok(map.get(&hash).expect("Just added should exist"))
    }
    pub fn hash_maven_id(group: &str, artifact: &str, version: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        (group, artifact, version).hash(&mut hasher);
        let hash = hasher.finish();
        hash
    }
    pub fn hash_maven_bom_id(group: &str, artifact: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        (group, artifact).hash(&mut hasher);
        let hash = hasher.finish();
        hash
    }
    pub fn resolve_properties_inital(pom: &mut MavenPom) {
        Self::resolve_properties(pom, resolve_string);
    }
    pub fn resolve_properties_final(pom: &mut MavenPom) {
        Self::resolve_properties(pom, resolve_string_final);
    }
    fn resolve_properties(
        pom: &mut MavenPom,
        mut resolver: impl FnMut(&mut String, &HashMap<String, String>),
    ) {
        debug!(
            "Resolving properties for {}:{}:{}",
            pom.group_id.as_ref().unwrap_or(&"(Blank)".to_string()),
            pom.artifact_id,
            pom.version.as_ref().unwrap_or(&"(Blank)".to_string())
        );
        let props = &mut pom.properties;
        // resolve the properties of the properties
        let c = props.map.clone();
        for map_prop in props.map.values_mut() {
            resolver(map_prop, &c);
        }

        if let Some(ref mut version) = pom.version {
            resolver(version, &props.map);
        }
        if let Some(ref mut group_id) = pom.group_id {
            resolver(group_id, &props.map);
        }
        if let Some(ref mut packaging) = pom.packaging {
            resolver(packaging, &props.map);
        }

        // Resolve properties in dependency management
        if let Some(ref mut dep_mgmt) = pom.dependency_management {
            for dep in &mut dep_mgmt.dependencies.dependency {
                if let Some(ref mut version) = dep.version {
                    resolver(version, &props.map);
                }
            }
        }

        // Resolve properties in dependencies
        if let Some(ref mut deps) = pom.dependencies {
            for dep in &mut deps.dependency {
                if let Some(ref mut version) = dep.version {
                    resolver(version, &props.map);
                }
            }
        }

        //resolve the dependency_management_map
        for map_value in pom.dependency_management_map.values_mut() {
            resolver(map_value, &props.map);
        }
    }
}
fn resolve_string(label: &mut String, map: &HashMap<String, String>) {
    static PROPERTY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\$\{(?<properties>\S+)\}").expect("Property Regex is not valid")
    });

    let mut replaced = label.to_string();
    for matches in PROPERTY_REGEX.captures_iter(&label) {
        let name = matches.name("properties");
        if let Some(capture) = name {
            if let Some(property) = map.get(capture.as_str()) {
                replaced = replaced.replace(&format!("${{{}}}", capture.as_str()), property);
            }
        }
    }

    *label = replaced;
}
fn resolve_string_final(label: &mut String, map: &HashMap<String, String>) {
    static PROPERTY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\$\{(?<properties>\S+)\}").expect("Property Regex is not valid")
    });

    let mut replaced = label.to_string();
    for matches in PROPERTY_REGEX.captures_iter(&label) {
        let name = matches.name("properties");
        if let Some(capture) = name {
            if let Some(property) = map.get(capture.as_str()) {
                replaced = replaced.replace(&format!("${{{}}}", capture.as_str()), property);
            } else {
                warn!(
                    "Property found in field but not present in map, property: {}",
                    capture.as_str()
                );
            }
        }
    }

    *label = replaced;
}
