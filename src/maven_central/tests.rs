#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::maven_central::{MavenId, fetch_artifact_metadata, fetch_jar, fetch_pom};

    fn client() -> Client {
        Client::new()
    }

    // Tests for fetch_pom()
    #[tokio::test]
    async fn test_get_pom_valid_artifact() {
        let result = fetch_pom(client(), &MavenId::new("junit", "junit", "4.13.2")).await;
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

    #[tokio::test]
    async fn test_get_pom_invalid_version() {
        let result = fetch_pom(client(), &MavenId::new("junit", "junit", "999.999.999")).await;
        assert!(result.is_err(), "Should fail for non-existent version");
    }

    #[tokio::test]
    async fn test_get_pom_invalid_artifact() {
        let result = fetch_pom(
            client(),
            &MavenId::new("junit", "nonexistent-artifact-xyz", "1.0.0"),
        )
        .await;
        assert!(result.is_err(), "Should fail for non-existent artifact");
    }

    #[tokio::test]
    async fn test_get_pom_with_dependencies() {
        let result = fetch_pom(
            client(),
            &MavenId::new("com.google.guava", "guava", "31.1-jre"),
        )
        .await;
        assert!(
            result.is_ok(),
            "Failed to fetch POM with dependencies: {:?}",
            result.err()
        );

        let pom = result.unwrap();
        assert_eq!(pom.group_id, "com.google.guava");
        assert_eq!(pom.artifact_id, "guava");
        assert!(pom.dependencies.is_some(), "Guava should have dependencies");
    }

    // Tests for fetch_jar()
    #[tokio::test]
    async fn test_get_jar_valid_artifact() {
        let result = fetch_jar(client(), &MavenId::new("junit", "junit", "4.13.2")).await;
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

    #[tokio::test]
    async fn test_get_jar_invalid_version() {
        let result = fetch_jar(client(), &MavenId::new("junit", "junit", "999.999.999")).await;
        assert!(result.is_err(), "Should fail for non-existent JAR version");
    }

    #[tokio::test]
    async fn test_get_jar_invalid_artifact() {
        let result = fetch_jar(
            client(),
            &MavenId::new("junit", "nonexistent-artifact-xyz", "1.0.0"),
        )
        .await;
        assert!(result.is_err(), "Should fail for non-existent JAR artifact");
    }

    #[tokio::test]
    async fn test_get_jar_small_vs_large() {
        let junit_result = fetch_jar(client(), &MavenId::new("junit", "junit", "4.13.2")).await;
        assert!(junit_result.is_ok());
        let junit_size = junit_result.unwrap().len();

        let guava_result = fetch_jar(
            client(),
            &MavenId::new("com.google.guava", "guava", "31.1-jre"),
        )
        .await;
        assert!(guava_result.is_ok());
        let guava_size = guava_result.unwrap().len();

        assert!(
            guava_size > junit_size,
            "Guava ({} bytes) should be larger than JUnit ({} bytes)",
            guava_size,
            junit_size
        );
    }

    // Tests for fetch_artifact_metadata()
    #[test]
    fn test_get_artifact_metadata_valid() {
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
        let result = fetch_artifact_metadata("junit", "junit");
        assert!(result.is_ok());

        let metadata = result.unwrap();
        let versions = &metadata.versioning.versions.version;
        assert!(
            versions.contains(&"4.13.2".to_string()),
            "Should contain version 4.13.2"
        );
    }

    #[test]
    fn test_get_artifact_metadata_invalid_artifact() {
        let result = fetch_artifact_metadata("junit", "nonexistent-artifact-xyz");
        assert!(
            result.is_err(),
            "Should fail for non-existent artifact metadata"
        );
    }

    #[test]
    fn test_get_artifact_metadata_invalid_group() {
        let result = fetch_artifact_metadata("nonexistent.group.xyz", "artifact");
        assert!(
            result.is_err(),
            "Should fail for non-existent group metadata"
        );
    }

    // Integration tests
    #[tokio::test]
    async fn test_get_pom_and_jar_workflow() {
        let cl = client();
        let pom = fetch_pom(cl.clone(), &MavenId::new("junit", "junit", "4.13.2"))
            .await
            .unwrap();
        let jar = fetch_jar(
            cl,
            &MavenId::new(&pom.group_id, &pom.artifact_id, &pom.version),
        )
        .await
        .unwrap();

        assert!(!jar.is_empty(), "JAR should be non-empty");
        assert!(jar.len() > 1000, "JAR should be reasonably sized");
    }

    #[test]
    fn test_get_metadata_and_pom_workflow() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let metadata = fetch_artifact_metadata("junit", "junit").unwrap();
        let latest_version = &metadata.versioning.latest;
        assert!(!latest_version.is_empty(), "Should have latest version");

        let pom = rt
            .block_on(fetch_pom(
                client(),
                &MavenId::new("junit", "junit", latest_version),
            ))
            .unwrap();
        assert_eq!(pom.version, *latest_version);
    }

    #[tokio::test]
    async fn test_get_pom_with_parent() {
        let result = fetch_pom(
            client(),
            &MavenId::new("com.fasterxml.jackson.core", "jackson-databind", "2.15.0"),
        )
        .await;
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
}
