use reqwest::blocking::Response;

use crate::{create_maven_url, maven_central::maven_error::MavenError};

fn full_url(group: &str, artifact: &str, version: &str, ext: &str) -> String {
    let url = format!(
        "{}{}/{}-{}.{}",
        create_maven_url(group, artifact),
        version,
        artifact,
        version,
        ext
    );
    log::debug!("Full Maven URL: {}", url);
    url
}
pub fn get_from_maven(
    group: &str,
    artifact: &str,
    version: &str,
    ext: &str,
) -> Result<Response, MavenError> {
    log::debug!("Fetching Maven artifact: {}:{}:{} ({})", group, artifact, version, ext);
    let url = full_url(group, artifact, version, ext);

    let res = reqwest::blocking::get(url)?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(MavenError::ErrorResponse(err))
        },
        Ok(res) => {
            log::debug!("Successfully fetched Maven artifact");
            Ok(res)
        }
    }
}
