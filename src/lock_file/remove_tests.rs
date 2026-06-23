#[cfg(test)]
mod tests {
    use crate::{
        lock_file::{LockFile, LockFilePackage},
        maven_central::MavenIdBuf,
    };

    fn package(id: &str, deps: &[&str]) -> LockFilePackage {
        let parts: Vec<&str> = id.split(':').collect();
        let dep_ids: Vec<MavenIdBuf> = deps
            .iter()
            .map(|d| {
                let p: Vec<&str> = d.split(':').collect();
                MavenIdBuf::new(p[0], p[1], p[2])
            })
            .collect();

        LockFilePackage {
            id: MavenIdBuf::new(parts[0], parts[1], parts[2]),
            file_name: String::new(),
            url: String::new(),
            dependancies: dep_ids,
        }
    }

    fn lockfile(packages: Vec<LockFilePackage>) -> LockFile {
        LockFile { packages }
    }

    #[test]
    fn transitive_chain() {
        let mut lock = lockfile(vec![
            package("g:a:1.0", &["g:b:1.0"]),
            package("g:b:2.0", &["g:c:1.0"]),
            package("g:c:3.0", &[]),
        ]);

        lock.remove_package("g", "a", true).unwrap();

        assert!(lock.packages.is_empty());
    }

    #[test]
    fn no_transitive_keeps_dependants() {
        let mut lock = lockfile(vec![
            package("g:a:1.0", &["g:b:1.0"]),
            package("g:b:2.0", &[]),
        ]);

        lock.remove_package("g", "a", false).unwrap();

        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].id.artifact, "b");
    }

    #[test]
    fn shared_dependency_transitive() {
        let mut lock = lockfile(vec![
            package("g:a:1.0", &["g:b:1.0", "g:c:1.0"]),
            package("g:b:2.0", &["g:c:1.0"]),
            package("g:c:3.0", &[]),
        ]);

        lock.remove_package("g", "a", true).unwrap();

        assert!(lock.packages.is_empty());
    }

    #[test]
    fn unrelated_package_survives_no_transitive() {
        let mut lock = lockfile(vec![
            package("g:a:1.0", &["g:b:1.0"]),
            package("g:b:2.0", &[]),
            package("g:d:4.0", &[]),
        ]);

        lock.remove_package("g", "a", false).unwrap();

        let artifacts: Vec<&str> =
            lock.packages.iter().map(|p| p.id.artifact.as_str()).collect();
        assert_eq!(artifacts, vec!["b", "d"]);
    }

    #[test]
    fn package_not_found() {
        let mut lock = lockfile(vec![package("g:a:1.0", &[])]);

        let result = lock.remove_package("g", "nonexistent", true);

        assert!(result.is_err());
        assert_eq!(lock.packages.len(), 1);
    }

    #[test]
    fn transitive_cleans_dependancy_lists() {
        let mut lock = lockfile(vec![
            package("g:x:1.0", &["g:a:1.0"]),
            package("g:a:1.0", &["g:b:1.0", "g:c:1.0"]),
            package("g:b:2.0", &["g:c:1.0"]),
            package("g:c:3.0", &[]),
        ]);

        lock.remove_package("g", "x", true).unwrap();

        assert!(lock.packages.is_empty());
    }
}
