use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::AsRefStr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "project")]
pub struct MavenPom {
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,

    #[serde(rename = "groupId", default)]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    #[serde(rename = "version", default)]
    pub version: String,

    #[serde(default)]
    pub packaging: DependancyType,

    pub dependencies: Option<Dependencies>,

    #[serde(
        rename = "dependencyManagement",
        skip_serializing_if = "Option::is_none"
    )]
    pub dependency_management: Option<DependencyManagement>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub properties: Properties,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Parent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<Build>,

    #[serde(skip)]
    pub dependency_management_map: HashMap<u64, String>,
}

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct Dependencies {
    #[serde(rename = "dependency", default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct DependencyManagement {
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependency {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(default)]
    pub scope: Scope,

    #[serde(default)]
    pub optional: bool,

    #[serde(rename = "type", default)]
    pub dependency_type: DependancyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifier: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct Properties {
    pub map: HashMap<String, String>,
}
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash, Default, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    #[default]
    Compile,

    Runtime,
    Provided,
    Test,
    System,
    Import,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum DependancyType {
    #[default]
    Jar,
    War,
    Pom,
    Bundle,

    Other(String),
}
impl Serialize for DependancyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for DependancyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(match s.as_str() {
            "jar" => DependancyType::Jar,
            "war" => DependancyType::War,
            "pom" => DependancyType::Pom,
            "bundle" => DependancyType::Bundle,
            _ => DependancyType::Other(s), // Catches absolutely any other string safely
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parent {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    pub version: String,

    pub relative_path: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Build {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Plugins>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Plugins {
    #[serde(rename = "plugin", default)]
    pub plugin: Vec<Plugin>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Plugin {
    #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId", skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Configuration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(
        rename = "annotationProcessorPaths",
        skip_serializing_if = "Option::is_none"
    )]
    pub annotation_processor_paths: Option<AnnotationProcessorPaths>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationProcessorPaths {
    #[serde(rename = "path", default)]
    pub path: Vec<ProcessorPath>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessorPath {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub version: String,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
