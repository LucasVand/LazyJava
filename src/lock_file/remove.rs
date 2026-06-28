use std::{
    collections::{HashMap, HashSet, VecDeque},
    mem,
};

use log::debug;

use crate::{
    lock_file::{LockFile, LockFileError, LockFilePackage},
    maven_central::MavenIdBuf,
};

impl LockFile {
    pub fn remove_package(
        &mut self,
        group: &str,
        artifact: &str,
    ) -> Result<LockFilePackage, LockFileError> {
        let pos = self.packages.iter().position(|v| {
            log::debug!("Checking {} against {}:{}", v.id, group, artifact);
            v.id.group == group && v.id.artifact == artifact
        });

        if let Some(pos) = pos {
            let package = self.packages.remove(pos);
            debug!("Removed package {}", package.id);

            self.packages.iter_mut().for_each(|p| {
                p.dependancies.retain(|dep| dep != &package.id);
            });

            Ok(package)
        } else {
            Err(LockFileError::PackageNotFound)
        }
    }

    pub fn remove_unneed_packages(&mut self) {
        let packages = mem::take(&mut self.packages);
        let n = packages.len();
        if n == 0 {
            return;
        }

        let id_to_idx: HashMap<&MavenIdBuf, usize> = packages
            .iter()
            .enumerate()
            .map(|(i, p)| (&p.id, i))
            .collect();

        let mut out_edges: Vec<Vec<usize>> = Vec::with_capacity(n);
        let mut in_degree = vec![0usize; n];

        for (i, p) in packages.iter().enumerate() {
            if p.root {
                in_degree[i] += 1;
            }
            let edges: Vec<usize> = p
                .dependancies
                .iter()
                .filter_map(|dep| id_to_idx.get(dep).copied())
                .collect();
            for &j in &edges {
                in_degree[j] += 1;
            }
            out_edges.push(edges);
        }

        let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut removed = vec![false; n];

        while let Some(i) = queue.pop_front() {
            log::info!("Removing unused package: {}", packages[i].id);
            removed[i] = true;
            for &j in &out_edges[i] {
                in_degree[j] -= 1;
                if in_degree[j] == 0 {
                    queue.push_back(j);
                }
            }
        }

        self.packages = packages
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !removed[*i])
            .map(|(_, p)| p)
            .collect();

        let remaining_ids: HashSet<MavenIdBuf> =
            self.packages.iter().map(|p| p.id.clone()).collect();
        for p in &mut self.packages {
            p.dependancies.retain(|dep| remaining_ids.contains(dep));
        }
    }
}
