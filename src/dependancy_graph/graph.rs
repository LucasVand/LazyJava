use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{IMPORT_REGEX, PACKAGE_REGEX};

#[derive(Debug, Clone)]
pub struct DependancyGraph {
    nodes: HashMap<String, DependancyNode>,
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
        let mut list = Vec::new();
        let node = self.nodes.get(id).unwrap();
        for dependant in &node.dependants {
            let resolved_node = self.nodes.get(dependant).unwrap();
            list.push(resolved_node.path.clone());

            list.append(&mut self.dependancy_list(&resolved_node.id));
        }
        return list;
    }

    pub fn calculate_dependants(&mut self) {
        let mut edges: Vec<(String, String)> = Vec::new();
        for (key, node) in &self.nodes {
            for dependancy in &node.dependancies {
                edges.push((dependancy.to_string(), key.to_string()));
            }
        }

        for edge in edges {
            let node = self.nodes.get_mut(&edge.0);
            if let Some(node) = node {
                node.dependants.push(edge.1);
            }
        }
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
