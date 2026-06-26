use crate::{
    create_maven_url,
    maven_central::{MavenError, MavenId, metadata::MavenMetadata},
};

pub fn fetch_artifact_metadata(group: &str, artifact: &str) -> Result<MavenMetadata, MavenError> {
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

pub fn full_maven_url(id: &MavenId, ext: &str) -> String {
    let url = format!(
        "{}{}/{}-{}.{}",
        create_maven_url(id.group, id.artifact),
        id.version,
        id.artifact,
        id.version,
        ext
    );
    log::debug!("Full Maven URL: {}", url);
    url
}
