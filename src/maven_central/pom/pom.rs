use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
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

    #[serde(rename = "dependencyManagement")]
    pub dependency_management: Option<DependencyManagement>,

    #[serde(default)]
    pub properties: Properties,

    pub parent: Option<Parent>,

    #[serde(skip)]
    pub dependency_management_map: HashMap<u64, String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Dependencies {
    #[serde(rename = "dependency", default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DependencyManagement {
    #[serde(default)]
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

    #[serde(default)]
    pub optional: bool,

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, Serialize)]
pub enum DependancyType {
    #[default]
    Jar,
    War,
    Pom,
    Bundle,

    Other(String),
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

#[derive(Debug, Deserialize)]
pub struct Parent {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    pub version: String,

    pub relative_path: Option<String>,
}
