use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    dependancy_graph::{
        graph::{DependancyGraph, DependancyNode},
        graph_error::GraphError,
    },
    utils::find_main::find_java_files,
};

impl DependancyGraph {
    pub fn create(src: &Path) -> Result<DependancyGraph, GraphError> {
        log::debug!("Creating dependency graph from source: {:?}", src);
        let java_files = find_java_files(src).map_err(|e| return GraphError::CreationError(e))?;
        log::debug!("Found {} Java files for graph", java_files.len());

        let mut graph = DependancyGraph::new();

        for file in java_files {
            let node = DependancyNode::from_file(&file)
                .map_err(|e| return GraphError::CreationError(e))?;

            graph.add_node(node);
        }
        log::debug!("Processing package dependencies");
        graph.calculate_package_dependancies();
        log::debug!("Processing dependants");
        graph.calculate_dependants();
        
        log::debug!("Dependency graph created with {} nodes", graph.nodes.len());
        return Ok(graph);
    }
    fn calculate_dependants(&mut self) {
        log::debug!("Calculating dependants for all nodes");
        let mut edges: Vec<(String, String)> = Vec::new();
        for (key, node) in &self.nodes {
            for dependancy in &node.dependancies {
                edges.push((dependancy.to_string(), key.to_string()));
            }
        }

        log::debug!("Processing {} edges for dependants", edges.len());
        for edge in edges {
            let node = self.nodes.get_mut(&edge.0);
            if let Some(node) = node {
                node.dependants.push(edge.1);
            }
        }
    }
    fn calculate_package_dependancies(&mut self) {
        log::debug!("Calculating package dependencies");
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for (_key, node) in &self.nodes {
            map.entry(node.package_name.clone())
                .or_default()
                .push(node.id.clone());
        }

        log::debug!("Processing {} package groups", map.len());
        self.nodes = std::mem::take(&mut self.nodes)
            .into_iter()
            .map(|(key, mut node)| {
                if node.package_name == "" {
                    return (key, node);
                }

                let package_files = map.get_mut(&node.package_name);
                if let Some(package_files) = package_files {
                    let mut hashset: HashSet<String> = node.dependancies.into_iter().collect();
                    package_files.iter().for_each(|file| {
                        // dont add ourself into the dependancy list
                        if *file != *node.id {
                            hashset.insert(file.clone());
                        }
                    });

                    node.dependancies = hashset.into_iter().collect();
                    return (key, node);
                }
                return (key, node);
            })
            .collect();
    }
}
