use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, LazyLock},
};

use log::{debug, warn};
use regex::{Regex, RegexBuilder};

use crate::maven_central::{
    MavenError, get_pom,
    pom::pom::{MavenPom, Scope},
};

pub struct MavenDependancyList {
    pub poms: HashMap<u64, MavenPom>,
}

enum PomState {
    Resolved(Arc<MavenPom>),
    Resolving,
}

type Cache = HashMap<u64, PomState>;

impl MavenDependancyList {
    pub fn new(group: &str, artifact: &str, version: &str) -> Result<Self, MavenError> {
        log::info!("Creating POM tree for {}:{}:{}", group, artifact, version);

        let mut cache = HashMap::new();
        Self::resolve_related_poms(group, artifact, version, &mut cache)?;

        log::info!("POM list created with {} total POMs", cache.len());
        Ok(MavenDependancyList {
            poms: cache
                .into_iter()
                .map(|(k, v)| match v {
                    PomState::Resolved(pom) => (
                        k,
                        Arc::into_inner(pom).expect("all references should be gone"),
                    ),
                    PomState::Resolving => panic!("Should have resolved all"),
                })
                .collect(),
        })
    }
    fn resolve_related_poms<'a>(
        group: &str,
        artifact: &str,
        version: &str,
        cache: &'a mut Cache,
    ) -> Result<Option<Arc<MavenPom>>, MavenError> {
        log::debug!("Resolving POM for {}:{}:{}", group, artifact, version);
        let hash = Self::hash_maven_id(group, artifact, version);

        if let Some(pom) = cache.get(&hash) {
            log::debug!(
                "Cache hit POM with hash: {} -> {}:{}:{}",
                hash,
                group,
                artifact,
                version
            );
            return match pom {
                PomState::Resolving => Ok(None),
                PomState::Resolved(pom) => Ok(Some(Arc::clone(pom))),
            };
        }

        let mut pom = get_pom(group, artifact, version)?;

        log::debug!(
            "(Cache Miss) Fetched POM with hash: {} -> {}:{}:{}",
            hash,
            group,
            artifact,
            version
        );
        cache.insert(hash, PomState::Resolving);

        Self::resolve_properties_inital(&mut pom);

        if let Some(parent) = &pom.parent {
            log::debug!(
                "Found parent POM: {}:{}:{}",
                parent.group_id,
                parent.artifact_id,
                parent.version
            );
            if let Some(parent_pom) = Self::resolve_related_poms(
                &parent.group_id,
                &parent.artifact_id,
                &parent.version,
                cache,
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
                if let Some(bom_pom) =
                    Self::resolve_related_poms(&dep.group_id, &dep.artifact_id, version, cache)?
                {
                    // extend properties
                    let mut bom_props = bom_pom.properties.map.clone();
                    bom_props.extend(pom.properties.map);
                    pom.properties.map = bom_props;
                    //
                    // // extends boms map
                    // let mut bom_boms = bom_pom.dependency_management_map.clone();
                    // bom_boms.extend(pom.dependency_management_map);
                    // pom.dependency_management_map = bom_boms;

                    // backwords
                    pom.dependency_management_map
                        .extend(bom_pom.dependency_management_map.clone());
                }
            }
        }
        Self::resolve_properties_inital(&mut pom);

        if let Some(deps) = &pom.dependencies {
            for dep in &deps.dependency {
                let scope = &dep.scope;
                if ![Scope::Compile, Scope::Runtime].contains(&scope) {
                    continue;
                }

                let version = dep.version.as_ref().unwrap_or_else(|| {
                    let dep_bom_hash = Self::hash_maven_bom_id(&dep.group_id, &dep.artifact_id);
                    let found_version = pom.dependency_management_map.get(&dep_bom_hash);

                    debug!(
                        "Found versioning in Bom for {}:{}, version: {}",
                        &dep.group_id,
                        &dep.artifact_id,
                        found_version.unwrap_or(&"(Blank)".to_string())
                    );
                    found_version.expect(&format!(
                        "Expected to find version in bom list, pom: {}:{}:{}, hash: {}. bom list: {:#?}",
                        group, artifact,version, 
                        hash, pom.dependency_management_map
                    ))
                });

                log::debug!(
                    "Resolving transitive dependency: {}:{}:{}",
                    dep.group_id,
                    dep.artifact_id,
                    version
                );
                if let Some(dep_pom) =
                    Self::resolve_related_poms(&dep.group_id, &dep.artifact_id, &version, cache)?
                {
                    let mut dep_props = dep_pom.properties.map.clone();
                    dep_props.extend(pom.properties.map);
                    pom.properties.map = dep_props;
                }
            }
        }
        Self::resolve_properties_final(&mut pom);

        let arc_pom = Arc::new(pom);
        let arc_pom_clone = arc_pom.clone();
        cache.insert(hash, PomState::Resolved(arc_pom));

        Ok(Some(arc_pom_clone))
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
