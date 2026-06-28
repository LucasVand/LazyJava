use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::maven_central::MavenIdBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ConfigProject,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependancies: Vec<ConfigDependancy>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigProject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigDependancy {
    pub id: MavenIdBuf,
}

impl From<MavenIdBuf> for ConfigDependancy {
    fn from(value: MavenIdBuf) -> Self {
        ConfigDependancy { id: value }
    }
}

impl<'de> Deserialize<'de> for ConfigDependancy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let split = s.split(":");
        let strings = split.into_iter().map(|s| s.to_string());

        let mut group: Option<String> = None;
        let mut artifact: Option<String> = None;
        let mut version: Option<String> = None;

        for string in strings.into_iter() {
            if group.is_none() {
                group = Some(string);
            } else if artifact.is_none() {
                artifact = Some(string);
            } else if version.is_none() {
                version = Some(string);
            }
        }

        if group.is_none() {
            return Err(Error::missing_field(
                "The Group is missing from the artifact declaration,, expected (Group):(Artifact):(Version)",
            ));
        }
        if artifact.is_none() {
            return Err(Error::missing_field(
                "The Artifact is missing from the artifact declaration, expected (Group):(Artifact):(Version)",
            ));
        }

        if version.is_none() {
            return Err(Error::missing_field(
                "The Artifact is missing from the artifact declaration, expected (Group):(Artifact):(Version)",
            ));
        }

        let id = MavenIdBuf::new(group.unwrap(), artifact.unwrap(), version.unwrap());
        return Ok(ConfigDependancy { id: id });
    }
}

impl Serialize for ConfigDependancy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!(
            "{}:{}:{}",
            self.id.group, self.id.artifact, self.id.version
        ))
    }
}
