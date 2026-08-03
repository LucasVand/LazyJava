mod build_error;
pub mod build_java;
pub mod find_stale_files;
pub(crate) mod metadata;

mod build_jar;
#[cfg(test)]
mod build_jar_tests;
mod compile;
#[cfg(test)]
mod compile_tests;
mod dependancy_graph;
mod graph;
#[cfg(test)]
mod metadata_tests;
mod processors;
mod resources;

pub use build_error::BuildError;
pub use dependancy_graph::GraphError;
