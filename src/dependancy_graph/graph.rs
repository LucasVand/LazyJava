use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{IMPORT_REGEX, PACKAGE_REGEX, dependancy_graph::graph_error::GraphError};

#[derive(Debug, Clone)]
pub struct DependancyGraph {
    pub nodes: HashMap<String, DependancyNode>,
}

#[derive(Debug, Clone)]
pub struct DependancyNode {
    pub path: PathBuf,
    pub file_name: String,
    pub package_name: String,
    pub dependancies: Vec<String>,
    pub id: String,
    pub dependants: Vec<String>,
}

impl DependancyGraph {
    pub fn new() -> DependancyGraph {
        DependancyGraph {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: DependancyNode) {
        self.nodes.insert(node.id.to_string(), node);
    }
    pub fn dependancy_list(&self, id: &str) -> Vec<PathBuf> {
        self.dependancy_list_internal(id, &mut Vec::new())
    }
    pub fn dependancy_list_from_path(&self, path: &Path) -> Result<Vec<PathBuf>, GraphError> {
        let id = self.find_id(path)?;

        return Ok(self.dependancy_list_internal(&id, &mut Vec::new()));
    }

    fn dependancy_list_internal(&self, id: &str, visited: &mut Vec<String>) -> Vec<PathBuf> {
        let mut list = Vec::new();
        let node = self.nodes.get(id).unwrap();
        for dependant in &node.dependants {
            let resolved_node = self.nodes.get(dependant).unwrap();
            list.push(resolved_node.path.clone());

            if visited.contains(dependant) {
                continue;
            }

            visited.push(id.to_string());
            list.append(&mut self.dependancy_list_internal(&resolved_node.id, visited));
        }
        return list;
    }
    fn find_id(&self, path: &Path) -> Result<String, GraphError> {
        let con_path = fs::canonicalize(path)?;
        for (key, node) in self.nodes.iter() {
            let con_node = fs::canonicalize(&node.path)?;

            if con_node == con_path {
                return Ok(key.clone());
            }
        }

        return Err(GraphError::NotFound(path.to_path_buf()));
    }
}

impl DependancyNode {
    pub fn from_file(path: &Path) -> Result<DependancyNode, io::Error> {
        let contents = fs::read_to_string(path)?;

        let matches = IMPORT_REGEX.captures_iter(&contents);
        let mut dependancies: Vec<String> = Vec::new();
        let package_match = PACKAGE_REGEX.captures(&contents);

        let package = if let Some(package) = package_match {
            package.name("package").unwrap().as_str().to_string()
        } else {
            "".to_string()
        };

        for item in matches {
            let import = item.name("import").unwrap();
            dependancies.push(import.as_str().to_string());
        }
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        let id = if package == "" {
            file_name.strip_suffix(".java").unwrap().to_string()
        } else {
            format!("{}.{}", package, file_name.strip_suffix(".java").unwrap())
        };

        let node = DependancyNode {
            path: path.to_path_buf(),
            file_name: file_name,
            package_name: package,
            dependancies: dependancies,
            id: id,
            dependants: Vec::new(),
        };

        return Ok(node);
    }
}
