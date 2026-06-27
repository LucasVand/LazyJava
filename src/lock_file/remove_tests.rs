#[cfg(test)]
mod tests {
use crate::{
    lock_file::{LockFile, LockFilePackage},
    maven_central::{MavenIdBuf, pom::DependancyType},
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
        packaging: DependancyType::Jar,
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

        lock.remove_package("g", "a").unwrap();

        assert!(lock.packages.is_empty());
    }



    #[test]
    fn shared_dependency_transitive() {
        let mut lock = lockfile(vec![
            package("g:a:1.0", &["g:b:1.0", "g:c:1.0"]),
            package("g:b:2.0", &["g:c:1.0"]),
            package("g:c:3.0", &[]),
        ]);

        lock.remove_package("g", "a").unwrap();

        assert!(lock.packages.is_empty());
    }



    #[test]
    fn package_not_found() {
        let mut lock = lockfile(vec![package("g:a:1.0", &[])]);

        let result = lock.remove_package("g", "nonexistent");

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

        lock.remove_package("g", "x").unwrap();

        assert!(lock.packages.is_empty());
    }
}
