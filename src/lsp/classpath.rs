use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "classpath")]
pub struct Classpath {
    // Treat repeated <classpathentry> tags as a Vector
    #[serde(rename = "classpathentry", default)]
    pub entries: Vec<ClasspathEntry>,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub struct ClasspathEntry {
    // Use #[serde(rename = "@...")] to map to XML attributes
    #[serde(rename = "@kind")]
    pub kind: String,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(rename = "@including")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub including: Option<String>, // Option because it's not on every entry
    #[serde(rename = "@output")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    // Nested <attributes> element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Attributes {
    // Nested list of <attribute> tags
    #[serde(rename = "attribute", default)]
    pub list: Vec<Attribute>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Attribute {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}
