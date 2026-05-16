use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "project")]
pub struct MavenPom {
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,

    #[serde(rename = "groupId")]
    pub group_id: Option<String>,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    #[serde(rename = "version")]
    pub version: Option<String>,

    pub packaging: Option<String>,

    pub dependencies: Option<Dependencies>,

    #[serde(rename = "dependencyManagement")]
    pub dependency_management: Option<DependencyManagement>,

    #[serde(default)]
    pub properties: Properties,

    pub parent: Option<Parent>,

    #[serde(skip)]
    pub dependency_management_map: HashMap<u64, String>,
}

#[derive(Debug, Deserialize)]
pub struct Dependencies {
    #[serde(rename = "dependency", default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
pub struct DependencyManagement {
    pub dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    pub version: Option<String>,

    #[serde(default)]
    pub scope: Scope,

    pub optional: Option<bool>,

    #[serde(rename = "type", default)]
    pub dependency_type: DependancyType,

    pub classifier: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Properties {
    pub map: HashMap<String, String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum DependancyType {
    #[default]
    Jar,
    War,
    Pom,

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Parent {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    pub version: String,

    pub relative_path: Option<String>,
}
