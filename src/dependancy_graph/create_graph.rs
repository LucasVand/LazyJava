use std::{io, path::Path};

use crate::{
    dependancy_graph::graph::{DependancyGraph, DependancyNode},
    utils::find_main::find_java_files,
};

pub fn create_dependancy_graph(src: &Path) -> Result<DependancyGraph, io::Error> {
    let java_files = find_java_files(src)?;

    let mut graph = DependancyGraph::new();

    for file in java_files {
        let node = DependancyNode::from_file(&file)?;

        graph.add_node(node);
    }
    graph.calculate_dependants();

    return Ok(graph);
}
