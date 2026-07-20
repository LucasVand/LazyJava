use crate::config::processor_list_serde::deserialize_processors;
use crate::config::processor_list_serde::serialize_processors;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    config::config_custom_serde::{deserialize_dependencies, serialize_dependencies},
    maven_central::PartialMavenIdBuf,
};
use decompose::decompose;
use serde::{Deserialize, Serialize};

use crate::maven_central::MavenIdBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub project: ConfigProject,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub setup: ConfigSetup,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub resources: ConfigResources,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    #[serde(
        serialize_with = "serialize_processors",
        deserialize_with = "deserialize_processors"
    )]
    pub processors: Vec<ConfigProcesserDefinition>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(
        serialize_with = "serialize_dependencies",
        deserialize_with = "deserialize_dependencies"
    )]
    pub dependancies: HashMap<PartialMavenIdBuf, ConfigDependancy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[decompose(
    ConfigProcesserDefinitionEntry,
    exclude(class_name),
    derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq),
    refs,
    ref_derive(Debug, Clone, Serialize, PartialEq, Eq)
)]
pub struct ConfigProcesserDefinition {
    pub class_name: String,
    pub kind: ProcesserType,
    pub path: PathBuf,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcesserType {
    Annotation,
    Processor,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigProject {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigSetup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct ConfigDependancy {
    pub id: MavenIdBuf,
}

impl From<MavenIdBuf> for ConfigDependancy {
    fn from(value: MavenIdBuf) -> Self {
        ConfigDependancy { id: value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigResources {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
