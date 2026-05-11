use crate::{
    create_maven_url,
    maven_central::{maven_error::MavenError, metadata::metadata::MavenMetadata},
};

// https://repo1.maven.org/maven2/GROUP/PATH/ARTIFACT/VERSION/FILE

pub fn get_artifact_metadata(group: &str, artifact: &str) -> Result<MavenMetadata, MavenError> {
    let url = create_maven_url(group, artifact);
    let full_url = format!("{}{}", url, "maven-metadata.xml");
    let request = reqwest::blocking::get(full_url)?;

    match request.error_for_status() {
        Err(err) => Err(MavenError::ErrorResponse(err.status().unwrap_or_default())),
        Ok(req) => {
            let meta_str = req.text()?;

            Ok(quick_xml::de::from_str(&meta_str)?)
        }
    }
}
