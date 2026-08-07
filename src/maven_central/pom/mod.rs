mod dependency_list;
mod dependency_list_structs;
mod pom;
mod pom_deserialize;
mod pom_properties_deserializer;

pub use dependency_list_structs::MavenDependencyList;
pub use pom::*;

#[cfg(test)]
mod tests;
