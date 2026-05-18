use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MavenMetadata {
    #[serde(rename = "groupId")]
    pub group_id: String,

    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    #[serde(default)]
    pub versioning: Versioning,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Versions {
    #[serde(rename = "version", default)]
    pub version: Vec<String>,
}

impl MavenMetadata {
    pub fn get_latest_version(&self) -> Option<&str> {
        if !self.versioning.release.is_empty() {
            log::debug!("Latest release version: {}", self.versioning.release);
            Some(&self.versioning.release)
        } else if !self.versioning.latest.is_empty() {
            log::debug!("Latest version: {}", self.versioning.latest);
            Some(&self.versioning.latest)
        } else {
            log::warn!(
                "No release or latest version found for {}:{}",
                self.group_id,
                self.artifact_id
            );
            None
        }
    }

    pub fn get_all_versions(&self) -> &[String] {
        log::debug!(
            "Available versions for {}:{}: {}",
            self.group_id,
            self.artifact_id,
            self.versioning.versions.version.len()
        );
        &self.versioning.versions.version
    }
}
