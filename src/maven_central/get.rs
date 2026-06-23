use crate::{
    create_maven_url,
    maven_central::{
        MavenError, MavenId, get_maven::get_from_maven, metadata::MavenMetadata, pom::MavenPom,
    },
};

pub fn get_pom(id: &MavenId) -> Result<MavenPom, MavenError> {
    log::info!("Fetching POM for {}", id);
    let str = get_from_maven(id, "pom")?.text()?;

    MavenPom::deserialize(id, &str)
}
pub fn get_jar(id: &MavenId) -> Result<Vec<u8>, MavenError> {
    log::info!("Fetching JAR for {}", id);
    let bin = get_from_maven(id, "jar")?;

    let bytes = bin.bytes()?.to_vec();
    log::debug!("JAR downloaded: {} bytes", bytes.len());
    Ok(bytes)
}
pub fn get_artifact_metadata(group: &str, artifact: &str) -> Result<MavenMetadata, MavenError> {
    log::info!("Fetching metadata for {}:{}", group, artifact);
    let url = create_maven_url(group, artifact);
    let full_url = format!("{}{}", url, "maven-metadata.xml");
    log::debug!("Metadata URL: {}", full_url);

    let request = reqwest::blocking::get(full_url)?;

    match request.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch metadata: {}", err);
            Err(MavenError::ErrorResponse(err))
        }
        Ok(req) => {
            let meta_str = req.text()?;
            log::debug!("Parsing metadata XML");

            let metadata = quick_xml::de::from_str(&meta_str)?;
            log::info!(
                "Successfully parsed metadata, found releases: {:?}",
                if let Ok(m) = quick_xml::de::from_str::<MavenMetadata>(&meta_str) {
                    m.versioning.versions.version.len()
                } else {
                    0
                }
            );
            Ok(metadata)
        }
    }
}
