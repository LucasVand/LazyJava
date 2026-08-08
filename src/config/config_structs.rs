use crate::{
    config::ConfigError,
    lock_file::{LocalRootPackage, RootPackage},
    maven_central::pom::Scope,
    utils::{IOError, fs::canonicalize},
};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

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
    pub processors: HashMap<String, ConfigProcessorDefinition>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, ConfigDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TomlEdit)]
#[serde(deny_unknown_fields)]
pub struct ConfigProcessorDefinition {
    pub kind: ProcessorType,
    pub path: PathBuf,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TomlEdit)]
#[serde(rename_all = "lowercase")]
pub enum ProcessorType {
    Annotation,
    Processor,
}
impl<'a> ConfigProcessorDefinitionTomlEditView<'a> {
    pub fn to_processer_definition(&self) -> Result<ConfigProcessorDefinition, ConfigError> {
        let kind = assert(self.kind(), "kind")?;
        let path = assert(self.path(), "path")?;
        let package = assert(self.package(), "package")?;
        Ok(ConfigProcessorDefinition {
            kind,
            path,
            package,
        })
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

    /// JDK release version passed to `javac --release` (e.g. "17")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq, TomlEdit, Serialize, Deserialize)]
#[toml_edit(inline)]
pub struct ConfigDependency {
    pub group: String,
    pub version: String,
    pub scope: Scope,
    pub path: String,
}
pub type RemoteDependency = RootPackage;
pub type LocalDependency = LocalRootPackage;

impl<'a> ConfigDependencyTomlEditView<'a> {
    pub fn to_remote_dependency(&self) -> Result<Option<RemoteDependency>, ConfigError> {
        let group = self.group();
        let version = self.version();
        let scope = self.scope();
        let path = self.path();

        if group.is_none() && version.is_none() {
            return Ok(None);
        }
        assert_none(path, "path")?;

        let group = assert(group, "group")?;
        let version = assert(version, "version")?;

        Ok(Some(RemoteDependency {
            group,
            version,
            scope,
        }))
    }
    pub fn to_local_dependency(&self) -> Result<Option<LocalDependency>, ConfigError> {
        let group = self.group();
        let version = self.version();
        let scope = self.scope();
        let path = self.path();

        if path.is_none() {
            return Ok(None);
        }
        assert_none(group, "group")?;
        assert_none(version, "version")?;

        let path = assert(path, "path")?;

        let path = PathBuf::from(&path);

        if !path.exists() {
            return Err(ConfigError::LocalDependencyNotFound(path));
        }
        if path.extension() != Some(OsStr::new("jar")) {
            return Err(ConfigError::LocalDependencyNotJar(path));
        }

        let con = canonicalize(&path)
            .map_err(|e| IOError::new("resolving local dependency path", path, e))?;

        Ok(Some(LocalDependency {
            path: con,
            scope: scope,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TomlEdit)]
#[serde(deny_unknown_fields)]
pub struct ConfigResources {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub external: Vec<String>,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

fn assert<T>(value: Option<T>, name: &'static str) -> Result<T, ConfigError> {
    if let Some(v) = value {
        Ok(v)
    } else {
        Err(ConfigError::MissingValue(name))
    }
}

fn assert_none<T>(value: Option<T>, name: &'static str) -> Result<(), ConfigError> {
    if let Some(_v) = value {
        Err(ConfigError::UnexpectedValue(name))
    } else {
        Ok(())
    }
}
