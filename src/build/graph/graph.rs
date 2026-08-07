use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use globset::GlobSet;

use crate::build::graph::{node::NodeFile, package::Package};

use super::node::Node;

pub struct Graph {
    pub root: Node,
    pub dependents: HashMap<PathBuf, Vec<PathBuf>>,
    pub dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl Graph {
    pub fn from_path(path: &Path, excluded: &GlobSet) -> Result<Graph, io::Error> {
        let root = Node::from_path(path, excluded)?;

        let mut dependents: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        let mut all_dependencies: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        let mut package_to_path: HashMap<&Package, &Path> = HashMap::new();

        for file in root.files() {
            package_to_path.insert(&file.package, &file.path);
        }

        for file in root.files() {
            let mut path_list: Vec<PathBuf> = Vec::new();
            for dep in &file.dependencies {
                if dep.is_wildcard() {
                    let node = root.find_package(dep);
                    if let Some(Node::Directory { files, .. }) = node {
                        path_list.extend(files.iter().filter_map(|v| match v {
                            Node::Directory { .. } => None,
                            Node::File(file) => Some(file.path.clone()),
                        }));
                    }
                } else if let Some(path) = package_to_path.get(dep) {
                    path_list.push(path.to_path_buf());
                }
            }
            all_dependencies.insert(file.path.to_path_buf(), path_list);
        }
        for (path, deps) in all_dependencies.iter() {
            for dep in deps {
                let entry = dependents.entry(dep.to_path_buf());
                let value = entry.or_insert(Vec::new());
                value.push(path.to_path_buf());
            }
        }

        Ok(Graph {
            root,
            dependents,
            dependencies: all_dependencies,
        })
    }
    fn dependents_of(&self, path: &Path) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut stack: Vec<PathBuf> = self.dependents.get(path).cloned().unwrap_or_default();

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            result.push(current.clone());
            if let Some(next) = self.dependents.get(&current) {
                stack.extend(next.iter().cloned());
            }
        }

        result
    }
    pub fn stale_files(&self, time: SystemTime) -> Vec<PathBuf> {
        let mut modified: Vec<&NodeFile> = Vec::new();
        for file in self.root.files() {
            let mtime = file.meta.modified().expect("Must be supported");

            if mtime > time {
                modified.push(file);
            }
        }

        let mut stale: HashSet<PathBuf> = HashSet::new();

        for m in modified {
            let dependents = self.dependents_of(&m.path);
            stale.extend(dependents);
            stale.insert(m.path.clone());
        }

        stale.into_iter().collect()
    }
}
