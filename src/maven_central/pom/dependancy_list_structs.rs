use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{
    lock_file::LockFilePackage,
    maven_central::{
        MavenIdBuf,
        fetch::full_maven_url,
        pom::{DependancyType, MavenPom},
    },
};

pub struct MavenDependancyList {}

pub enum PomState {
    Resolved(Arc<MavenPom>),
    Resolving,
}

pub type Cache = Arc<RwLock<HashMap<u64, PomState>>>;
pub type DependancyList = Arc<RwLock<Vec<MavenDependancy>>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MavenDependancy {
    pub id: MavenIdBuf,
    pub dependancy_type: DependancyType,
    pub dependancies: Vec<Dependancy>,
    pub root: bool,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependancy {
    pub id: MavenIdBuf,
}

impl From<MavenDependancy> for LockFilePackage {
    fn from(value: MavenDependancy) -> Self {
        let ext = |t: &DependancyType| match t {
            DependancyType::Jar => "jar",
            DependancyType::War => "war",
            DependancyType::Bundle => "jar",
            _ => panic!("Unsupported dependancy type"),
        };
        let ext_str = ext(&value.dependancy_type);

        let url = full_maven_url(&value.id.as_maven_id(), ext_str);
        let file_name = format!("{}-{}.{}", &value.id.artifact, &value.id.version, ext_str);

        LockFilePackage {
            packaging: value.dependancy_type,
            id: value.id,
            url,
            file_name,
            dependancies: value.dependancies.into_iter().map(|v| v.id).collect(),
            root: value.root,
        }
    }
}
