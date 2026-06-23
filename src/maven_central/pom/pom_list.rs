use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, LazyLock},
};

use log::{debug, warn};
use maven_version::Maven3ArtifactVersion;
use regex::{Regex, RegexBuilder};

use crate::{
    lock_file::LockFilePackage,
    maven_central::{
        MavenError, MavenId, MavenIdBuf,
        get_maven::full_maven_url,
        get_pom,
        pom::pom::{DependancyType, MavenPom, Scope},
    },
};

pub struct MavenDependancyList {}

enum PomState {
    Resolved(Arc<MavenPom>),
    Resolving,
}

type Cache = HashMap<u64, PomState>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MavenDependancy {
    pub id: MavenIdBuf,
    pub dependancy_type: DependancyType,
    pub dependancies: Vec<Dependancy>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependancy {
    pub id: MavenIdBuf,
}

impl MavenDependancyList {
    pub fn new(id: &MavenId) -> Result<Vec<MavenDependancy>, MavenError> {
        log::info!("Creating POM list for {}", id);

        let mut cache = HashMap::new();
        let mut dep_list = Vec::new();
        Self::resolve_related_poms(id, &mut cache, &mut dep_list)?;

        let mut map: HashMap<u64, MavenDependancy> = HashMap::new();

        // takes the newest version
        for dep in dep_list.into_iter() {
            let hash = Self::hash_maven_bom_id(&dep.id.group, &dep.id.artifact);
            if let Some(lookup) = map.get(&hash) {
                let lookup_version = Maven3ArtifactVersion::new(&lookup.id.version);
                let new_version = Maven3ArtifactVersion::new(&dep.id.version);

                if new_version > lookup_version {
                    map.insert(hash, dep);
                }
            } else {
                map.insert(hash, dep);
            }
        }

        log::info!("POM list created with {} total POMs", cache.len());
        Ok(map.into_iter().map(|(_k, v)| v).collect())
    }
    fn resolve_related_poms<'a>(
        id: &MavenId,
        cache: &'a mut Cache,
        list: &mut Vec<MavenDependancy>,
    ) -> Result<Option<Arc<MavenPom>>, MavenError> {
        log::debug!("Resolving POM for {}", id);
        let hash = Self::hash_maven_id(id);

        if let Some(pom) = cache.get(&hash) {
            log::debug!("Cache hit POM with hash: {} -> {}", hash, id);
            return match pom {
                PomState::Resolving => Ok(None),
                PomState::Resolved(pom) => Ok(Some(Arc::clone(pom))),
            };
        }

        let mut pom = get_pom(id)?;

        log::debug!("(Cache Miss) Fetched POM with hash: {} -> {}", hash, id);
        cache.insert(hash, PomState::Resolving);

        Self::resolve_properties_inital(&mut pom);

        if let Some(parent) = &pom.parent {
            log::debug!(
                "Found parent POM: {}:{}:{} for {}",
                parent.group_id,
                parent.artifact_id,
                parent.version,
                id
            );
            if let Some(parent_pom) = Self::resolve_related_poms(
                &MavenId::new(&parent.group_id, &parent.artifact_id, &parent.version),
                cache,
                list,
            )? {
                let mut parent_props = parent_pom.properties.map.clone();
                parent_props.extend(pom.properties.map);

                pom.properties.map = parent_props;

                // backwords for right now
                pom.dependency_management_map
                    .extend(parent_pom.dependency_management_map.clone());

                Self::resolve_properties_inital(&mut pom);
            }
        }

        if let Some(dep_management) = &pom.dependency_management {
            for dep in &dep_management.dependencies.dependency {
                let scope = &dep.scope;
                if *scope != Scope::Import || dep.optional {
                    continue;
                }
                let bom_version = dep.version.as_ref().expect("Bom is missing version");

                log::debug!(
                    "Found BOM import: {}:{}:{} for {}",
                    dep.group_id,
                    dep.artifact_id,
                    bom_version,
                    id
                );
                if let Some(bom_pom) = Self::resolve_related_poms(
                    &MavenId::new(&dep.group_id, &dep.artifact_id, bom_version),
                    cache,
                    list,
                )? {
                    // extend properties
                    let mut bom_props = bom_pom.properties.map.clone();
                    bom_props.extend(pom.properties.map);
                    pom.properties.map = bom_props;

                    // backwords
                    pom.dependency_management_map
                        .extend(bom_pom.dependency_management_map.clone());
                }
            }
        }
        Self::resolve_properties_inital(&mut pom);

        // tracks the dep list for the dependancy list
        let mut dependancy_list: Vec<Dependancy> = Vec::new();
        if let Some(deps) = &pom.dependencies {
            for dep in &deps.dependency {
                let scope = &dep.scope;
                if ![Scope::Compile, Scope::Runtime].contains(&scope) || dep.optional {
                    continue;
                }

                let dep_version = dep.version.as_ref().unwrap_or_else(|| {
                    let dep_bom_hash = Self::hash_maven_bom_id(&dep.group_id, &dep.artifact_id);
                    let found_version = pom.dependency_management_map.get(&dep_bom_hash);

                    debug!(
                        "Found versioning in Bom for {}:{}, version: {}",
                        &dep.group_id,
                        &dep.artifact_id,
                        found_version.unwrap_or(&"(Blank)".to_string())
                    );
                    found_version.expect(&format!(
                        "Expected to find version in bom list, pom: {}, hash: {}. bom list: {:#?}",
                        id, hash, pom.dependency_management_map
                    ))
                });

                log::debug!(
                    "Resolving transitive dependency: {}:{}:{} for {}",
                    dep.group_id,
                    dep.artifact_id,
                    dep_version,
                    id,
                );
                if let Some(dep_pom) = Self::resolve_related_poms(
                    &MavenId::new(&dep.group_id, &dep.artifact_id, &dep_version),
                    cache,
                    list,
                )? {
                    let mut dep_props = dep_pom.properties.map.clone();
                    dep_props.extend(pom.properties.map);
                    pom.properties.map = dep_props;

                    dependancy_list.push(Dependancy {
                        id: MavenIdBuf::new(
                            dep.group_id.clone(),
                            dep.artifact_id.clone(),
                            dep_version.clone(),
                        ),
                    });
                }
            }
        }
        Self::resolve_properties_final(&mut pom);

        if pom.packaging != DependancyType::Pom && pom.packaging != DependancyType::Other {
            list.push(MavenDependancy {
                id: MavenIdBuf::new(id.group, id.artifact, id.version),
                dependancy_type: pom.packaging,
                dependancies: dependancy_list,
            });
        }

        let arc_pom = Arc::new(pom);
        let arc_pom_clone = arc_pom.clone();
        cache.insert(hash, PomState::Resolved(arc_pom));

        Ok(Some(arc_pom_clone))
    }
    pub fn hash_maven_id(id: &MavenId) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
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
            pom.group_id, pom.artifact_id, pom.version,
        );
        let props = &mut pom.properties;
        // resolve the properties of the properties
        let c = props.map.clone();
        for map_prop in props.map.values_mut() {
            resolver(map_prop, &c);
        }

        resolver(&mut pom.version, &props.map);
        resolver(&mut pom.group_id, &props.map);

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
                resolver(&mut dep.group_id, &props.map);
            }
        }

        //resolve the dependency_management_map
        for map_value in pom.dependency_management_map.values_mut() {
            resolver(map_value, &props.map);
        }
    }
}
static PROPERTY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\$\{(?<properties>\S+)\}")
        .swap_greed(true)
        .build()
        .expect("Property Regex is not valid")
});
fn resolve_string(label: &mut String, map: &HashMap<String, String>) {
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

impl From<MavenDependancy> for LockFilePackage {
    fn from(value: MavenDependancy) -> Self {
        if value.dependancy_type != DependancyType::Jar {
            panic!("Only jars are supported currently");
        }
        let url = full_maven_url(&value.id.as_maven_id(), "jar");
        let file_name = format!("{}-{}.{}", &value.id.artifact, &value.id.version, "jar");

        LockFilePackage {
            group: value.id.group,
            artifact: value.id.artifact,
            version: value.id.version,
            url,
            file_name,
            dependancies: value
                .dependancies
                .into_iter()
                .map(|v| (v.id.group, v.id.artifact, v.id.version))
                .collect(),
        }
    }
}
