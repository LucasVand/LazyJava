use log::warn;

use crate::maven_central::{
    MavenError,
    pom::{
        pom::{DependancyType, Scope},
        pom_tree::MavenDependancyList,
    },
};

fn get_maven_dependancies_internal(
    group: &str,
    artifact: &str,
    version: &str,
    list: &MavenDependancyList,
) -> Vec<MavenDependancy> {
    let hash = MavenDependancyList::hash_maven_id(group, artifact, version);
    let mut deps = Vec::new();

    let pom = list.poms.get(&hash);
    if let Some(pom) = pom {
        if let Some(dep_list) = &pom.dependencies {
            for dep in dep_list.dependency.iter() {
                let scope = &dep.scope;
                if ![Scope::Compile, Scope::Runtime].contains(&scope) {
                    continue;
                }
                let version = dep.version.clone().unwrap();

                let mut inner_dependancies = get_maven_dependancies_internal(
                    &dep.group_id,
                    &dep.artifact_id,
                    &version,
                    list,
                );

                deps.append(&mut inner_dependancies);
                deps.push(MavenDependancy {
                    group: dep.group_id.clone(),
                    artifact: dep.artifact_id.clone(),
                    version: version.to_string(),
                    scope: *scope,
                    dependancy_type: dep.dependency_type,
                });
                log::debug!("Added dependency: {}:{}", dep.group_id, dep.artifact_id);
            }
        }
    } else {
        warn!(
            "Attempted to get pom but does not exist in tree. pom {}:{}:{}",
            group, artifact, version
        );
    }

    deps
}

pub fn get_maven_dependancies(
    group: &str,
    artifact: &str,
    version: &str,
) -> Result<Vec<MavenDependancy>, MavenError> {
    let pom_list = MavenDependancyList::new(group, artifact, version)?;
    let deps = get_maven_dependancies_internal(group, artifact, version, &pom_list);

    return Ok(deps.into_iter().collect());
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MavenDependancy {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub scope: Scope,
    pub dependancy_type: DependancyType,
}
