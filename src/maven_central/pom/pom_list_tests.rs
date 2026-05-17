#[cfg(test)]
mod tests {

    use crate::maven_central::pom::{
        MavenDependancyList, pom::DependancyType, pom_list::MavenDependancy,
    };
    #[test]
    fn init_logger() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .init();
    }

    #[test]
    fn test_maven_dependancy_list_simple() {
        // Test creating a dependency list for a simple artifact (junit has no dependencies)
        let result = MavenDependancyList::new("junit", "junit", "4.13.2");
        assert!(
            result.is_ok(),
            "Failed to create dependency list: {:?}",
            result.err()
        );

        let dep_list = result.unwrap();
        // JUnit itself should be in the list (as a non-POM artifact)
        assert!(!dep_list.is_empty(), "Dependency list should contain junit");

        let junit_dep = dep_list
            .iter()
            .find(|d| d.group == "junit" && d.artifact == "junit" && d.version == "4.13.2");
        assert!(
            junit_dep.is_some(),
            "JUnit 4.13.2 should be in dependency list"
        );
    }

    #[test]
    fn test_maven_dependancy_list_excludes_pom_packaging() {
        // Test that POM-type dependencies are excluded from the list
        let result =
            MavenDependancyList::new("com.fasterxml.jackson.core", "jackson-databind", "2.15.0");
        assert!(
            result.is_ok(),
            "Failed to create dependency list: {:?}",
            result.err()
        );

        let dep_list = result.unwrap();

        // All dependencies should be JAR, not POM
        for dep in &dep_list {
            assert!(
                dep.dependancy_type != DependancyType::Pom,
                "POM dependencies should be excluded from list, found: {}:{}",
                dep.group,
                dep.artifact
            );
        }
    }

    #[test]
    fn test_maven_dependancy_list_invalid_artifact() {
        // Test that invalid artifacts produce errors
        let result = MavenDependancyList::new("invalid.group", "invalid.artifact", "1.0.0");
        assert!(result.is_err(), "Should fail for non-existent artifact");
    }

    #[test]
    fn test_maven_dependancy_list_against_real_list() {
        // Test that transitive dependencies are included
        let result = MavenDependancyList::new("com.google.guava", "guava", "33.6.0-jre");
        assert!(result.is_ok());

        let mut dep_list = result.unwrap();

        let mut expected = vec![
            MavenDependancy {
                group: "com.google.errorprone".into(),
                artifact: "error_prone_annotations".into(),
                version: "2.47.0".into(),
                dependancy_type: DependancyType::Jar,
            },
            MavenDependancy {
                group: "com.google.guava".into(),
                artifact: "failureaccess".into(),
                version: "1.0.3".into(),
                dependancy_type: DependancyType::Jar,
            },
            MavenDependancy {
                group: "com.google.guava".into(),
                artifact: "listenablefuture".into(),
                version: "9999.0-empty-to-avoid-conflict-with-guava".into(),
                dependancy_type: DependancyType::Jar,
            },
            MavenDependancy {
                group: "com.google.j2objc".into(),
                artifact: "j2objc-annotations".into(),
                version: "3.1".into(),
                dependancy_type: DependancyType::Jar,
            },
            MavenDependancy {
                group: "org.jspecify".into(),
                artifact: "jspecify".into(),
                version: "1.0.0".into(),
                dependancy_type: DependancyType::Jar,
            },
        ];
        expected.sort();
        dep_list.sort();

        for (expected, result) in expected.iter().zip(dep_list.iter()) {
            assert!(
                expected == result,
                "Resulting and expected dependences do not match, expected: {:#?}, result: {:#?}",
                expected,
                result
            );
        }
    }

    #[test]
    fn test_maven_dependancy_list_no_duplicates() {
        // Test that the list doesn't have duplicate entries
        let result = MavenDependancyList::new("com.google.guava", "guava", "33.6.0-jre");
        assert!(result.is_ok());

        let dep_list = result.unwrap();
        let original_len = dep_list.len();

        // Create a set to check for duplicates
        let mut unique_deps = std::collections::HashSet::new();
        let mut has_duplicates = false;

        for dep in &dep_list {
            if !unique_deps.insert((dep.group.clone(), dep.artifact.clone(), dep.version.clone())) {
                has_duplicates = true;
                println!(
                    "Found duplicate: {}:{}:{}",
                    dep.group, dep.artifact, dep.version
                );
            }
        }

        assert!(
            !has_duplicates,
            "Dependency list should not have duplicates (original: {}, unique: {})",
            original_len,
            unique_deps.len()
        );
    }

    #[test]
    fn test_maven_dependancy_list_versions_valid() {
        // Test that all versions in the list are valid version strings
        let result = MavenDependancyList::new("com.google.guava", "guava", "31.1-jre");
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        for dep in &dep_list {
            assert!(
                !dep.version.is_empty(),
                "Dependency {}:{} has empty version",
                dep.group,
                dep.artifact
            );
            assert!(!dep.group.is_empty(), "Dependency has empty group");
            assert!(!dep.artifact.is_empty(), "Dependency has empty artifact");
        }
    }

    #[test]
    fn test_maven_dependancy_list_root_artifact_included() {
        // Test that the root artifact is included in the dependency list
        let result = MavenDependancyList::new("junit", "junit", "4.13.2");
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        let root = dep_list
            .iter()
            .find(|d| d.group == "junit" && d.artifact == "junit" && d.version == "4.13.2");

        assert!(
            root.is_some(),
            "Root artifact should be in the dependency list"
        );
    }

    #[test]
    fn test_maven_dependancy_list_dependency_types() {
        // Test that dependencies have valid packaging types
        let result = MavenDependancyList::new("com.google.guava", "guava", "31.1-jre");
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        for dep in &dep_list {
            assert!(
                dep.dependancy_type == DependancyType::Jar
                    || dep.dependancy_type == DependancyType::Other,
                "Dependency {}:{} has invalid type: {:?}",
                dep.group,
                dep.artifact,
                dep.dependancy_type
            );
        }
    }
}
