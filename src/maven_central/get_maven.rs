use reqwest::blocking::Response;

use crate::{
    create_maven_url,
    maven_central::{MavenId, maven_error::MavenError},
};

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
pub fn get_from_maven(id: &MavenId, ext: &str) -> Result<Response, MavenError> {
    log::debug!("Fetching Maven artifact: {} ({})", id, ext);
    let url = full_maven_url(id, ext);

    let res = reqwest::blocking::get(url)?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(MavenError::ErrorResponse(err))
        }
        Ok(res) => Ok(res),
    }
}
