use crate::config::ConfigError;
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit_derive::TomlEdit;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TomlEdit)]
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
    pub processors: HashMap<String, ConfigProcesserDefinition>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependancies: HashMap<String, ConfigDependancy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TomlEdit)]
#[serde(deny_unknown_fields)]
pub struct ConfigProcesserDefinition {
    pub kind: ProcesserType,
    pub path: PathBuf,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TomlEdit)]
#[serde(rename_all = "lowercase")]
pub enum ProcesserType {
    Annotation,
    Processor,
}
impl<'a> ConfigProcesserDefinitionTomlEditView<'a> {
    pub fn to_processer_definition(&self) -> Result<ConfigProcesserDefinition, ConfigError> {
        let kind = assert(self.kind(), "kind")?;
        let path = assert(self.path(), "path")?;
        let package = assert(self.package(), "package")?;
        Ok(ConfigProcesserDefinition {
            kind,
            path,
            package,
        })
    }
}
fn assert<T>(value: Option<T>, name: &'static str) -> Result<T, ConfigError> {
    if let Some(v) = value {
        return Ok(v);
    } else {
        Err(ConfigError::MissingValue(name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TomlEdit)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TomlEdit)]
#[serde(deny_unknown_fields)]
pub struct ConfigSetup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, TomlEdit, Serialize, Deserialize)]
#[toml_edit(inline)]
pub struct ConfigDependancy {
    pub group: String,
    pub version: String,
}

impl<'a> ConfigDependancyTomlEditView<'a> {
    pub fn to_config_dependancy(&self) -> Result<ConfigDependancy, ConfigError> {
        let group = assert(self.group(), "group_id")?;
        let version = assert(self.version(), "version")?;
        Ok(ConfigDependancy {
            group: group,
            version: version,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TomlEdit)]
#[serde(deny_unknown_fields)]
pub struct ConfigResources {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
