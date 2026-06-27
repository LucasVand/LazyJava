#[cfg(test)]
mod tests {

    use crate::maven_central::{
        MavenIdBuf,
        pom::{MavenDependancyList, dependancy_list_structs::MavenDependancy, pom::DependancyType},
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
        let result = MavenDependancyList::new(MavenIdBuf::new("junit", "junit", "4.13.2"));
        assert!(
            result.is_ok(),
            "Failed to create dependency list: {:?}",
            result.err()
        );

        let dep_list = result.unwrap();
        // JUnit itself should be in the list (as a non-POM artifact)
        assert!(!dep_list.is_empty(), "Dependency list should contain junit");

        let junit_dep = dep_list.iter().find(|d| {
            d.id.group == "junit" && d.id.artifact == "junit" && d.id.version == "4.13.2"
        });
        assert!(
            junit_dep.is_some(),
            "JUnit 4.13.2 should be in dependency list"
        );
    }

    #[test]
    fn test_maven_dependancy_list_excludes_pom_packaging() {
        // Test that POM-type dependencies are excluded from the list
        let result = MavenDependancyList::new(MavenIdBuf::new(
            "com.fasterxml.jackson.core",
            "jackson-databind",
            "2.15.0",
        ));
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
                dep.id.group,
                dep.id.artifact
            );
        }
    }

    #[test]
    fn test_maven_dependancy_list_invalid_artifact() {
        // Test that invalid artifacts produce errors
        let result = MavenDependancyList::new(MavenIdBuf::new(
            "invalid.id.group",
            "invalid.id.artifact",
            "1.0.0",
        ));
        assert!(result.is_err(), "Should fail for non-existent artifact");
    }

    #[test]
    fn test_maven_dependancy_list_against_real_list() {
        // Test that transitive dependencies are included
        let result =
            MavenDependancyList::new(MavenIdBuf::new("com.google.guava", "guava", "33.6.0-jre"));
        assert!(result.is_ok());

        let mut dep_list = result.unwrap();

        let mut expected = vec![
            MavenDependancy {
                id: MavenIdBuf::new("com.google.guava", "guava", "33.6.0-jre"),
                dependancy_type: DependancyType::Bundle,
                dependancies: Vec::new(),
            },
            MavenDependancy {
                id: MavenIdBuf::new("com.google.errorprone", "error_prone_annotations", "2.47.0"),
                dependancy_type: DependancyType::Jar,
                dependancies: Vec::new(),
            },
            MavenDependancy {
                id: MavenIdBuf::new("com.google.guava", "failureaccess", "1.0.3"),
                dependancy_type: DependancyType::Jar,
                dependancies: Vec::new(),
            },
            MavenDependancy {
                id: MavenIdBuf::new(
                    "com.google.guava",
                    "listenablefuture",
                    "9999.0-empty-to-avoid-conflict-with-guava",
                ),
                dependancy_type: DependancyType::Jar,
                dependancies: Vec::new(),
            },
            MavenDependancy {
                id: MavenIdBuf::new("com.google.j2objc", "j2objc-annotations", "3.1"),
                dependancy_type: DependancyType::Jar,
                dependancies: Vec::new(),
            },
            MavenDependancy {
                id: MavenIdBuf::new("org.jspecify", "jspecify", "1.0.0"),
                dependancy_type: DependancyType::Jar,
                dependancies: Vec::new(),
            },
        ];
        expected.sort();
        dep_list.sort();

        for (expected, result) in expected.iter().zip(dep_list.iter()) {
            assert!(
                expected.id == result.id && expected.dependancy_type == result.dependancy_type,
                "Resulting and expected dependences do not match, expected: {:#?}, result: {:#?}",
                expected,
                result
            );
        }
    }

    #[test]
    fn test_maven_dependancy_list_no_duplicates() {
        // Test that the list doesn't have duplicate entries
        let result = MavenDependancyList::new(MavenIdBuf::new(
            "org.springframework.boot",
            "spring-boot-starter-web",
            "4.1.0-RC1",
        ));
        assert!(result.is_ok());

        let dep_list = result.unwrap();
        let original_len = dep_list.len();

        // Create a set to check for duplicates
        let mut unique_deps = std::collections::HashSet::new();
        let mut has_duplicates = false;

        for dep in &dep_list {
            if !unique_deps.insert((dep.id.group.clone(), dep.id.artifact.clone())) {
                has_duplicates = true;
                println!("Found duplicate: {}:{}", dep.id.group, dep.id.artifact);
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
        let result =
            MavenDependancyList::new(MavenIdBuf::new("com.google.guava", "guava", "31.1-jre"));
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        for dep in &dep_list {
            assert!(
                !dep.id.version.is_empty(),
                "Dependency {}:{} has empty version",
                dep.id.group,
                dep.id.artifact
            );
            assert!(!dep.id.group.is_empty(), "Dependency has empty group");
            assert!(!dep.id.artifact.is_empty(), "Dependency has empty artifact");
        }
    }

    #[test]
    fn test_maven_dependancy_list_root_artifact_included() {
        // Test that the root artifact is included in the dependency list
        let result = MavenDependancyList::new(MavenIdBuf::new("junit", "junit", "4.13.2"));
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        let root = dep_list.iter().find(|d| {
            d.id.group == "junit" && d.id.artifact == "junit" && d.id.version == "4.13.2"
        });

        assert!(
            root.is_some(),
            "Root artifact should be in the dependency list"
        );
    }

    #[test]
    fn test_maven_dependancy_list_dependency_types() {
        // Test that dependencies have valid packaging types
        let result =
            MavenDependancyList::new(MavenIdBuf::new("com.google.guava", "guava", "31.1-jre"));
        assert!(result.is_ok());

        let dep_list = result.unwrap();

        for dep in &dep_list {
            assert!(
                dep.dependancy_type == DependancyType::Jar
                    || dep.dependancy_type == DependancyType::Bundle
                    || matches!(dep.dependancy_type, DependancyType::Other(_)),
                "Dependency {}:{} has invalid type: {:?}",
                dep.id.group,
                dep.id.artifact,
                dep.dependancy_type
            );
        }
    }
}
