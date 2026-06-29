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
        MavenId {
            group,
            artifact,
            version,
        }
    }

    pub fn to_buf(&self) -> MavenIdBuf {
        MavenIdBuf {
            group: self.group.to_string(),
            artifact: self.artifact.to_string(),
            version: self.version.to_string(),
        }
    }
}

impl fmt::Display for MavenId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)
    }
}

impl From<MavenId<'_>> for MavenIdBuf {
    fn from(value: MavenId<'_>) -> Self {
        value.to_buf()
    }
}

impl<'a> From<&'a MavenId<'_>> for MavenIdBuf {
    fn from(value: &'a MavenId<'_>) -> Self {
        value.to_buf()
    }
}

impl<'a> From<(&'a str, &'a str, &'a str)> for MavenIdBuf {
    fn from((group, artifact, version): (&'a str, &'a str, &'a str)) -> Self {
        MavenIdBuf::new(group, artifact, version)
    }
}

impl From<(String, String, String)> for MavenIdBuf {
    fn from((group, artifact, version): (String, String, String)) -> Self {
        MavenIdBuf::new(group, artifact, version)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MavenIdBuf {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl MavenIdBuf {
    pub fn new(
        group: impl Into<String>,
        artifact: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        MavenIdBuf {
            group: group.into(),
            artifact: artifact.into(),
            version: version.into(),
        }
    }

    pub fn as_maven_id(&self) -> MavenId<'_> {
        MavenId::new(&self.group, &self.artifact, &self.version)
    }
    pub fn to_partial_buf(self) -> PartialMavenIdBuf {
        self.into()
    }
}

impl<'a> From<&'a MavenIdBuf> for MavenId<'a> {
    fn from(value: &'a MavenIdBuf) -> Self {
        value.as_maven_id()
    }
}

impl PartialEq<MavenIdBuf> for MavenId<'_> {
    fn eq(&self, other: &MavenIdBuf) -> bool {
        self.group == other.group
            && self.artifact == other.artifact
            && self.version == other.version
    }
}

impl PartialEq<MavenId<'_>> for MavenIdBuf {
    fn eq(&self, other: &MavenId<'_>) -> bool {
        other.eq(self)
    }
}

impl fmt::Display for MavenIdBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.artifact, self.version)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PartialMavenIdBuf {
    pub group: String,
    pub artifact: String,
}
impl PartialMavenIdBuf {
    pub fn new(group: impl Into<String>, artifact: impl Into<String>) -> PartialMavenIdBuf {
        PartialMavenIdBuf {
            group: group.into(),
            artifact: artifact.into(),
        }
    }
    pub fn to_full_buf(self, version: impl Into<String>) -> MavenIdBuf {
        MavenIdBuf::new(self.group, self.artifact, version.into())
    }
}
impl From<MavenIdBuf> for PartialMavenIdBuf {
    fn from(value: MavenIdBuf) -> Self {
        PartialMavenIdBuf {
            group: value.group,
            artifact: value.artifact,
        }
    }
}
