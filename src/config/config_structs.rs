use std::collections::HashMap;

use crate::{
    config::config_custom_serde::{deserialize_dependencies, serialize_dependencies},
    maven_central::PartialMavenIdBuf,
};
use serde::{Deserialize, Serialize};

use crate::maven_central::MavenIdBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ConfigProject,

    #[serde(default)]
    pub setup: ConfigSetup,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(
        serialize_with = "serialize_dependencies",
        deserialize_with = "deserialize_dependencies"
    )]
    pub dependancies: HashMap<PartialMavenIdBuf, ConfigDependancy>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConfigProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConfigSetup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ConfigDependancy {
    pub id: MavenIdBuf,
}

impl From<MavenIdBuf> for ConfigDependancy {
    fn from(value: MavenIdBuf) -> Self {
        ConfigDependancy { id: value }
    }
}
