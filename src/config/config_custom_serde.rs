use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::{
    config::ConfigDependancy,
    maven_central::{MavenIdBuf, PartialMavenIdBuf},
};

#[derive(Serialize, Deserialize)]
struct DependencyEntry {
    group: String,
    version: String,
}

pub fn serialize_dependencies<S>(
    deps: &HashMap<PartialMavenIdBuf, ConfigDependancy>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = HashMap::with_capacity(deps.len());

    for (_key, dep) in deps {
        let entry = DependencyEntry {
            group: dep.id.group.clone(),
            version: dep.id.version.clone(),
        };

        map.insert(&dep.id.artifact, entry);
    }

    map.serialize(serializer)
}

pub fn deserialize_dependencies<'de, D>(
    deserializer: D,
) -> Result<HashMap<PartialMavenIdBuf, ConfigDependancy>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, DependencyEntry> = HashMap::deserialize(deserializer)?;

    let mut deps = HashMap::with_capacity(map.len());

    for (artifact, entry) in map {
        if artifact.is_empty() {
            return Err(D::Error::custom("Artifact name cannot be empty"));
        }

        if entry.group.is_empty() {
            return Err(D::Error::custom(format!(
                "Dependency '{artifact}' is missing a group"
            )));
        }

        if entry.version.is_empty() {
            return Err(D::Error::custom(format!(
                "Dependency '{artifact}' is missing a version"
            )));
        }

        let id = MavenIdBuf::new(entry.group, artifact, entry.version);
        deps.insert(id.clone().into(), ConfigDependancy { id: id });
    }

    Ok(deps)
}
