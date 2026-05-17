use crate::maven_central::{
    MavenError,
    pom::{pom::DependancyType, pom_tree::MavenDependancyList},
};

pub fn get_maven_dependancies(
    group: &str,
    artifact: &str,
    version: &str,
) -> Result<Vec<MavenDependancy>, MavenError> {
    let pom_list = MavenDependancyList::new(group, artifact, version)?;

    return Ok(pom_list.dependencies);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MavenDependancy {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub dependancy_type: DependancyType,
}
