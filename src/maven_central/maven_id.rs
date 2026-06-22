use std::fmt;

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
