use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MavenId<'a> {
    pub group: &'a str,
    pub artifact: &'a str,
    pub version: &'a str,
}

impl<'a> MavenId<'a> {
    pub fn new(group: &'a str, artifact: &'a str, version: &'a str) -> Self {
        MavenId { group, artifact, version }
    }
}

impl fmt::Display for MavenId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MavenIdBuf {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl MavenIdBuf {
    pub fn new(group: impl Into<String>, artifact: impl Into<String>, version: impl Into<String>) -> Self {
        MavenIdBuf {
            group: group.into(),
            artifact: artifact.into(),
            version: version.into(),
        }
    }

    pub fn as_maven_id(&self) -> MavenId<'_> {
        MavenId::new(&self.group, &self.artifact, &self.version)
    }
}

impl<'a> From<&'a MavenIdBuf> for MavenId<'a> {
    fn from(value: &'a MavenIdBuf) -> Self {
        value.as_maven_id()
    }
}

impl fmt::Display for MavenIdBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)
    }
}
