use std::{collections::HashSet, fmt::write};

use log::warn;

use crate::maven_central::{
    MavenError, get_pom,
    pom::{
        pom::{DependancyType, Scope},
        pom_tree::MavenPomTree,
    },
};

fn get_maven_dependancies_raw(
    group: &str,
    artifact: &str,
    version: &str,
) -> Result<Vec<MavenDependancy>, MavenError> {
    log::debug!(
        "Resolving Maven dependencies for {}:{}:{}",
        group,
        artifact,
        version
    );

    let pom = get_pom(group, artifact, &version)?;
    dbg!(&pom);

    let mut dep_list: Vec<MavenDependancy> = Vec::new();
    if let Some(deps) = pom.dependencies {
        for dep in deps.dependency.into_iter() {
            let scope = dep.scope;
            if ![Scope::Compile, Scope::Runtime].contains(&scope) {
                continue;
            }
            let version = dep.version.unwrap();

            log::debug!(
                "Resolving transitive dependencies for {}:{}:{}",
                dep.group_id,
                dep.artifact_id,
                version
            );
            let mut inner_dependancies =
                get_maven_dependancies(&dep.group_id, &dep.artifact_id, &version)?;

            dep_list.append(&mut inner_dependancies);
            dep_list.push(MavenDependancy {
                group: dep.group_id.clone(),
                artifact: dep.artifact_id.clone(),
                version: version,
                scope: scope,
                dependancy_type: dep.dependency_type,
            });
            log::debug!("Added dependency: {}:{}", dep.group_id, dep.artifact_id);
        }
    } else {
        log::debug!("No dependencies found in POM");
    }

    if let Some(parent) = pom.parent {
        log::debug!("Resolving transitive dependencies for parent",);
        let mut parent_deps =
            get_maven_dependancies_raw(&parent.group_id, &parent.artifact_id, &parent.version)?;
        dep_list.append(&mut parent_deps);
    }
    log::debug!(
        "Found {} dependencies for {}:{}:{}",
        dep_list.len(),
        group,
        artifact,
        version
    );

    Ok(dep_list)
}
fn get_maven_dependancies_internal(
    group: &str,
    artifact: &str,
    version: &str,
    tree: &MavenPomTree,
) -> Vec<MavenDependancy> {
    let hash = MavenPomTree::hash_maven_id(group, artifact, version);
    let mut deps = Vec::new();

    let pom = tree.poms.get(&hash);
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
                    tree,
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
    let pom_tree = MavenPomTree::new(group.to_string(), artifact.to_string(), version.to_string())?;
    let deps = get_maven_dependancies_internal(group, artifact, version, &pom_tree);

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
