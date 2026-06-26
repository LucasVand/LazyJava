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
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependancy {
    pub id: MavenIdBuf,
}

impl From<MavenDependancy> for LockFilePackage {
    fn from(value: MavenDependancy) -> Self {
        if value.dependancy_type != DependancyType::Jar {
            panic!("Only jars are supported currently");
        }
        let url = full_maven_url(&value.id.as_maven_id(), "jar");
        let file_name = format!("{}-{}.{}", &value.id.artifact, &value.id.version, "jar");

        LockFilePackage {
            id: value.id,
            url,
            file_name,
            dependancies: value.dependancies.into_iter().map(|v| v.id).collect(),
        }
    }
}
