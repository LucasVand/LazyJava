use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::Notify;

use crate::{
    lock_file::LockFilePackageRemote,
    maven_central::{
        MavenIdBuf,
        fetch::full_maven_url,
        pom::{DependencyType, MavenPom, Scope},
    },
};

pub struct MavenDependencyList {}

pub enum PomState {
    Resolved(Arc<MavenPom>),
    Resolving(Arc<Notify>),
}

pub type Cache = Arc<RwLock<HashMap<u64, PomState>>>;
pub type DependencyList = Arc<RwLock<Vec<MavenDependency>>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MavenDependency {
    pub id: MavenIdBuf,
    pub dependency_type: DependencyType,
    pub dependencies: Vec<Dependency>,
    pub scope: Scope,
    pub root: bool,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    pub id: MavenIdBuf,
}

impl From<MavenDependency> for LockFilePackageRemote {
    fn from(value: MavenDependency) -> Self {
        let ext = |t: &DependencyType| match t {
            DependencyType::Jar => "jar",
            DependencyType::War => "war",
            DependencyType::Bundle => "jar",
            _ => panic!("Unsupported dependency type"),
        };
        let ext_str = ext(&value.dependency_type);

        let url = full_maven_url(&value.id.as_maven_id(), ext_str);
        let file_name = format!("{}-{}.{}", &value.id.artifact, &value.id.version, ext_str);

        LockFilePackageRemote {
            packaging: value.dependency_type,
            id: value.id,
            url,
            file_name,
            dependencies: value.dependencies.into_iter().map(|v| v.id).collect(),
            root: value.root,
            annotations: Vec::new(),
            scope: value.scope,
            path: None,
        }
    }
}
