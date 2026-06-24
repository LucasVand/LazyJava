#[cfg(test)]
mod tests {
    use crate::maven_central::{MavenId, fetch_artifact_metadata, fetch_jar, fetch_pom};

    // Tests for get_pom()
    #[test]
    fn test_get_pom_valid_artifact() {
        // Test fetching a real POM from Maven Central
        let result = fetch_pom(&MavenId::new("junit", "junit", "4.13.2"));
        assert!(
            result.is_ok(),
            "Failed to fetch valid POM: {:?}",
            result.err()
        );

        let pom = result.unwrap();
        assert_eq!(pom.group_id, "junit");
        assert_eq!(pom.artifact_id, "junit");
        assert_eq!(pom.version, "4.13.2");
    }

    #[test]
    fn test_get_pom_invalid_version() {
        // Test fetching a POM with invalid version
        let result = fetch_pom(&MavenId::new("junit", "junit", "999.999.999"));
        assert!(result.is_err(), "Should fail for non-existent version");
    }

    #[test]
    fn test_get_pom_invalid_artifact() {
        // Test fetching a POM with invalid artifact
        let result = fetch_pom(&MavenId::new("junit", "nonexistent-artifact-xyz", "1.0.0"));
        assert!(result.is_err(), "Should fail for non-existent artifact");
    }

    #[test]
    fn test_get_pom_with_dependencies() {
        // Test fetching a POM that has dependencies
        let result = fetch_pom(&MavenId::new("com.google.guava", "guava", "31.1-jre"));
        assert!(
            result.is_ok(),
            "Failed to fetch POM with dependencies: {:?}",
            result.err()
        );

        let pom = result.unwrap();
        assert_eq!(pom.group_id, "com.google.guava");
        assert_eq!(pom.artifact_id, "guava");
        // Guava should have dependencies
        assert!(pom.dependencies.is_some(), "Guava should have dependencies");
    }

    // Tests for get_jar()
    #[test]
    fn test_get_jar_valid_artifact() {
        // Test fetching a real JAR from Maven Central
        let result = fetch_jar(&MavenId::new("junit", "junit", "4.13.2"));
        assert!(
            result.is_ok(),
            "Failed to fetch valid JAR: {:?}",
            result.err()
        );

        let jar_bytes = result.unwrap();
        assert!(!jar_bytes.is_empty(), "JAR should not be empty");
        assert!(
            jar_bytes.len() > 1000,
            "JAR should be reasonably sized (got {} bytes)",
            jar_bytes.len()
        );

        // JAR files should start with PK (0x504B in hex)
        assert_eq!(jar_bytes[0], 0x50, "JAR should start with 'P'");
        assert_eq!(jar_bytes[1], 0x4B, "JAR should start with 'K'");
    }

    #[test]
    fn test_get_jar_invalid_version() {
        // Test fetching a JAR with invalid version
        let result = fetch_jar(&MavenId::new("junit", "junit", "999.999.999"));
        assert!(result.is_err(), "Should fail for non-existent JAR version");
    }

    #[test]
    fn test_get_jar_invalid_artifact() {
        // Test fetching a JAR with invalid artifact
        let result = fetch_jar(&MavenId::new("junit", "nonexistent-artifact-xyz", "1.0.0"));
        assert!(result.is_err(), "Should fail for non-existent JAR artifact");
    }

    #[test]
    fn test_get_jar_small_vs_large() {
        // Compare small and large JARs to verify size differences
        let junit_result = fetch_jar(&MavenId::new("junit", "junit", "4.13.2"));
        assert!(junit_result.is_ok());
        let junit_size = junit_result.unwrap().len();

        let guava_result = fetch_jar(&MavenId::new("com.google.guava", "guava", "31.1-jre"));
        assert!(guava_result.is_ok());
        let guava_size = guava_result.unwrap().len();

        // Guava should be larger than JUnit
        assert!(
            guava_size > junit_size,
            "Guava ({} bytes) should be larger than JUnit ({} bytes)",
            guava_size,
            junit_size
        );
    }

    // Tests for get_artifact_metadata()
    #[test]
    fn test_get_artifact_metadata_valid() {
        // Test fetching metadata for a real artifact
        let result = fetch_artifact_metadata("junit", "junit");
        assert!(
            result.is_ok(),
            "Failed to fetch valid metadata: {:?}",
            result.err()
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.group_id, "junit");
        assert_eq!(metadata.artifact_id, "junit");
        assert!(
            !metadata.versioning.versions.version.is_empty(),
            "Should have version list"
        );
    }

    #[test]
    fn test_get_artifact_metadata_latest_version() {
        // Test that metadata contains latest version info
        let result = fetch_artifact_metadata("junit", "junit");
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert!(
            !metadata.versioning.latest.is_empty(),
            "Should have latest version"
        );
        assert!(
            !metadata.versioning.release.is_empty(),
            "Should have release version"
        );
    }

    #[test]
    fn test_get_artifact_metadata_has_versions() {
        // Test that metadata contains specific versions
        let result = fetch_artifact_metadata("junit", "junit");
        assert!(result.is_ok());

        let metadata = result.unwrap();
        let versions = &metadata.versioning.versions.version;

        // JUnit 4.13.2 should be in the version list
        assert!(
            versions.contains(&"4.13.2".to_string()),
            "Should contain version 4.13.2"
        );
    }

    #[test]
    fn test_get_artifact_metadata_invalid_artifact() {
        // Test metadata fetch for non-existent artifact
        let result = fetch_artifact_metadata("junit", "nonexistent-artifact-xyz");
        assert!(
            result.is_err(),
            "Should fail for non-existent artifact metadata"
        );
    }

    #[test]
    fn test_get_artifact_metadata_invalid_group() {
        // Test metadata fetch for non-existent group
        let result = fetch_artifact_metadata("nonexistent.group.xyz", "artifact");
        assert!(
            result.is_err(),
            "Should fail for non-existent group metadata"
        );
    }

    #[test]
    fn test_get_artifact_metadata_large_project() {
        // Test metadata for a project with many versions
        let result = fetch_artifact_metadata("com.google.guava", "guava");
        assert!(result.is_ok());

        let metadata = result.unwrap();
        let version_count = metadata.versioning.versions.version.len();

        // Guava is a mature project, should have many versions
        assert!(
            version_count > 50,
            "Guava should have many versions (got {})",
            version_count
        );
    }

    // Integration tests combining multiple functions
    #[test]
    fn test_get_pom_and_jar_workflow() {
        // Test the typical workflow: fetch POM, then fetch JAR
        let pom_result = fetch_pom(&MavenId::new("junit", "junit", "4.13.2"));
        assert!(pom_result.is_ok());
        let pom = pom_result.unwrap();

        let jar_result = fetch_jar(&MavenId::new(&pom.group_id, &pom.artifact_id, &pom.version));
        assert!(jar_result.is_ok());
        let jar = jar_result.unwrap();

        assert!(!jar.is_empty(), "JAR should be non-empty");
        assert!(jar.len() > 1000, "JAR should be reasonably sized");
    }

    #[test]
    fn test_get_metadata_and_pom_workflow() {
        // Test the workflow: fetch metadata to get latest version, then fetch POM
        let metadata_result = fetch_artifact_metadata("junit", "junit");
        assert!(metadata_result.is_ok());
        let metadata = metadata_result.unwrap();

        let latest_version = &metadata.versioning.latest;
        assert!(!latest_version.is_empty(), "Should have latest version");

        let pom_result = fetch_pom(&MavenId::new("junit", "junit", latest_version));
        assert!(
            pom_result.is_ok(),
            "Should be able to fetch POM for latest version: {:?}",
            pom_result.err()
        );

        let pom = pom_result.unwrap();
        assert_eq!(pom.version, *latest_version);
    }

    #[test]
    fn test_get_pom_with_parent() {
        // Test fetching a POM that has a parent POM reference
        let result = fetch_pom(&MavenId::new(
            "com.fasterxml.jackson.core",
            "jackson-databind",
            "2.15.0",
        ));
        assert!(
            result.is_ok(),
            "Failed to fetch POM with parent: {:?}",
            result.err()
        );

        let pom = result.unwrap();
        assert!(
            pom.parent.is_some(),
            "jackson-databind should have a parent POM"
        );

        let parent = pom.parent.unwrap();
        assert_eq!(parent.group_id, "com.fasterxml.jackson");
        assert_eq!(parent.artifact_id, "jackson-base");
    }

    #[test]
    fn test_error_messages_are_descriptive() {
        // Verify that error messages are useful for debugging
        let result = fetch_pom(&MavenId::new("invalid.group", "invalid.artifact", "1.0.0"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(
            !error_str.is_empty(),
            "Error should have a descriptive message"
        );
    }
}
