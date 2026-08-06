#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use tempfile::tempdir;

    use crate::build::metadata::BuildMetadata;

    #[test]
    fn new_creates_default_metadata() {
        let meta = BuildMetadata::new();
        assert_eq!(meta.java_version, "");
        assert_eq!(meta.lib_hash, 0);
        assert_eq!(meta.bin_hash, 0);
        assert_eq!(meta.local_libs_hash, 0);
        assert!(!meta.build_passed);
    }

    #[test]
    fn fetch_returns_none_when_no_file() {
        let dir = tempdir().unwrap();
        assert!(BuildMetadata::fetch(dir.path()).is_none());
    }

    #[test]
    fn write_and_fetch_round_trip() {
        let dir = tempdir().unwrap();
        let meta = BuildMetadata {
            time_stamp: SystemTime::UNIX_EPOCH,
            java_version: "21".into(),
            src: "src".into(),
            lib_hash: 42,
            lib_annotations_hash: 7,
            bin_hash: 123,
            local_libs_hash: 99,
            build_passed: true,
        };

        meta.write(dir.path()).unwrap();
        let fetched = BuildMetadata::fetch(dir.path()).unwrap();

        assert_eq!(fetched.java_version, "21");
        assert_eq!(fetched.lib_hash, 42);
        assert_eq!(fetched.lib_annotations_hash, 7);
        assert_eq!(fetched.bin_hash, 123);
        assert_eq!(fetched.local_libs_hash, 99);
        assert!(fetched.build_passed);
    }

    #[test]
    fn hash_directory_is_deterministic() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        fs::write(dir.path().join("b.txt"), b"world").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/c.txt"), b"!").unwrap();

        let hash1 = crate::build::metadata::hash_directory(dir.path());
        let hash2 = crate::build::metadata::hash_directory(dir.path());
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_directory_changes_when_file_added() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let before = crate::build::metadata::hash_directory(dir.path());

        fs::write(dir.path().join("b.txt"), b"world").unwrap();
        let after = crate::build::metadata::hash_directory(dir.path());

        assert_ne!(before, after);
    }

    #[test]
    fn hash_directory_changes_when_file_removed() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        fs::write(dir.path().join("b.txt"), b"world").unwrap();
        let before = crate::build::metadata::hash_directory(dir.path());

        fs::remove_file(dir.path().join("b.txt")).unwrap();
        let after = crate::build::metadata::hash_directory(dir.path());

        assert_ne!(before, after);
    }

    #[test]
    fn hash_files_changes_when_content_changes() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("lib.jar");
        fs::write(&file, b"AAAA").unwrap();
        let before = crate::build::metadata::hash_files(&[file.clone()]);

        fs::write(&file, b"BBBB").unwrap();
        let after = crate::build::metadata::hash_files(&[file.clone()]);

        assert_ne!(before, after);
    }

    #[test]
    fn hash_files_is_deterministic() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("lib.jar");
        fs::write(&file, b"hello-world").unwrap();

        let hash1 = crate::build::metadata::hash_files(&[file.clone()]);
        let hash2 = crate::build::metadata::hash_files(&[file.clone()]);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_files_changes_when_file_added() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.jar");
        fs::write(&a, b"A").unwrap();
        let before = crate::build::metadata::hash_files(&[a.clone()]);

        let b = dir.path().join("b.jar");
        fs::write(&b, b"B").unwrap();
        let after = crate::build::metadata::hash_files(&[a.clone(), b.clone()]);

        assert_ne!(before, after);
    }

    #[test]
    fn hash_files_changes_when_file_removed() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.jar");
        fs::write(&a, b"A").unwrap();
        let b = dir.path().join("b.jar");
        fs::write(&b, b"B").unwrap();

        let before = crate::build::metadata::hash_files(&[a.clone(), b.clone()]);
        let after = crate::build::metadata::hash_files(&[a.clone()]);

        assert_ne!(before, after);
    }
}
