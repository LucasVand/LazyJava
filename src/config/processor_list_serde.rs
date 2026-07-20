use crate::config::ConfigProcesserDefinitionEntryRef;
use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::config::{
    ConfigProcesserDefinition, ConfigProcesserDefinitionEntry,
    ConfigProcesserDefinitionEntryExcluded,
};

pub fn serialize_processors<S>(
    procs: &Vec<ConfigProcesserDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map: HashMap<&String, ConfigProcesserDefinitionEntryRef> =
        HashMap::with_capacity(procs.len());

    for proc in procs {
        let (entry, ex) = proc.decompose_ref();

        map.insert(ex.class_name, entry);
    }

    map.serialize(serializer)
}

pub fn deserialize_processors<'de, D>(
    deserializer: D,
) -> Result<Vec<ConfigProcesserDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, ConfigProcesserDefinitionEntry> = HashMap::deserialize(deserializer)?;

    Ok(map
        .into_iter()
        .map(|(k, v)| {
            ConfigProcesserDefinition::compose(
                v,
                ConfigProcesserDefinitionEntryExcluded { class_name: k },
            )
        })
        .collect())
}
