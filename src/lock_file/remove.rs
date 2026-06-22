use std::{collections::HashMap, mem};

use log::debug;

use crate::{
    lock_file::{LockFile, LockFileError, LockFilePackage},
    maven_central::pom::MavenDependancyList,
    packages,
};

struct Node {
    out_edges: Vec<u64>,

    package: LockFilePackage,
}

impl LockFile {
    pub fn remove_package(
        &mut self,
        group: &str,
        artifact: &str,
        resolve_transitive: bool,
    ) -> Result<(), LockFileError> {
        let pos = self
            .packages
            .iter()
            .position(|v| &v.group == group && &v.artifact == artifact);

        if let Some(pos) = pos {
            let package = self.packages.remove(pos);
            debug!(
                "Removed package {}:{}:{}",
                package.group, package.artifact, package.version
            );

            //remove from all the dependancies here too this also sucks
            self.packages.iter_mut().for_each(|p| {
                p.dependancies.retain(|dep| {
                    &dep.0 != &p.group && &dep.1 != &p.artifact && &dep.2 != &p.version
                });
            });

            if resolve_transitive {
                debug!("Resolving unused packages");
                self.remove_unneed_packages();
            }
            Ok(())
        } else {
            Err(LockFileError::PackageNotFound)
        }
    }
    // this function is horrible
    fn remove_unneed_packages(&mut self) {
        loop {
            let mut map: HashMap<u64, Node> = mem::take(&mut self.packages)
                .into_iter()
                .map(|p| {
                    (
                        MavenDependancyList::hash_maven_id(&p.group, &p.artifact, &p.version),
                        Node {
                            out_edges: p
                                .dependancies
                                .iter()
                                .map(|v| MavenDependancyList::hash_maven_id(&v.0, &v.1, &v.2))
                                .collect(),
                            package: p,
                        },
                    )
                })
                .collect();

            // Count incoming edges for each package
            let mut in_degree: HashMap<u64, u64> = map.iter().map(|(k, _v)| (*k, 0_u64)).collect();

            for (_k, v) in &map {
                for edge in v.out_edges.iter() {
                    let count = in_degree.get_mut(edge).expect("Should exist");
                    *count += 1;
                }
            }

            let mut removed = false;

            // Find and remove packages with no incoming edges (nothing depends on them)
            let to_remove: Vec<u64> = in_degree
                .iter()
                .filter(|(_k, count)| **count == 0)
                .map(|(k, _)| *k)
                .collect();

            log::debug!("Found {} packages with no dependents", to_remove.len());

            for k in to_remove {
                if let Some(node) = map.remove(&k) {
                    log::info!(
                        "Removing unused package: {}:{}:{}",
                        node.package.group,
                        node.package.artifact,
                        node.package.version
                    );
                    removed = true;
                    // Remove edges from other packages pointing to the removed package
                    for (_key, v) in map.iter_mut() {
                        v.out_edges.retain(|dep| *dep != k);
                    }
                }
            }

            // this part could be better by making the node out edges more representative and then
            // just transforming them back to the string string string
            //
            // Convert back to packages, syncing the modified out_edges back to dependancies
            self.packages = map
                .into_iter()
                .map(|(_k, mut node)| {
                    // Rebuild dependancies from the updated out_edges
                    let remaining_deps: Vec<(String, String, String)> = node
                        .out_edges
                        .iter()
                        .filter_map(|edge_hash| {
                            // Find the package that matches this hash
                            node.package
                                .dependancies
                                .iter()
                                .find(|dep| {
                                    MavenDependancyList::hash_maven_id(&dep.0, &dep.1, &dep.2)
                                        == *edge_hash
                                })
                                .cloned()
                        })
                        .collect();

                    node.package.dependancies = remaining_deps;
                    node.package
                })
                .collect();

            if !removed {
                log::debug!("No more unused packages to remove");
                return;
            }
        }
    }
}
