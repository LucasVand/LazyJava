use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MavenMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    #[serde(default)]
    pub versioning: Versioning,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Versioning {
    #[serde(default)]
    pub latest: String,

    #[serde(default)]
    pub release: String,

    #[serde(default)]
    pub versions: Versions,

    #[serde(rename = "lastUpdated", default)]
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Versions {
    #[serde(rename = "version", default)]
    pub version: Vec<String>,
}
