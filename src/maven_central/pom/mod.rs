mod pom;
mod pom_deserialize;
mod pom_list;
mod properties_deserializer;

pub use pom::*;
pub use pom_list::MavenDependancyList;

#[cfg(test)]
mod pom_list_tests;
