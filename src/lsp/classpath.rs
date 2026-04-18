use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Classpath {
    // Treat repeated <classpathentry> tags as a Vector
    #[serde(rename = "classpathentry", default)]
    pub entries: Vec<ClasspathEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ClasspathEntry {
    // Use #[serde(rename = "@...")] to map to XML attributes
    #[serde(rename = "@kind")]
    pub kind: String,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(rename = "@including")]
    pub including: Option<String>, // Option because it's not on every entry
    #[serde(rename = "@output")]
    pub output: Option<String>,

    // Nested <attributes> element
    pub attributes: Option<Attributes>,
}

#[derive(Debug, Deserialize)]
pub struct Attributes {
    // Nested list of <attribute> tags
    #[serde(rename = "attribute", default)]
    pub list: Vec<Attribute>,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}
