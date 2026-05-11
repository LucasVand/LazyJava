use crate::{create_maven_url, maven_central::maven_error::MavenError};

fn full_url(group: &str, artifact: &str, version: &str) -> String {
    format!(
        "{}{}/{}-{}.jar",
        create_maven_url(group, artifact),
        version,
        artifact,
        version
    )
}
pub fn get_jar(group: &str, artifact: &str, version: &str) -> Result<Vec<u8>, MavenError> {
    let url = full_url(group, artifact, version);

    let res = reqwest::blocking::get(url)?;

    match res.error_for_status() {
        Err(err) => Err(MavenError::ErrorResponse(err.status().unwrap_or_default())),
        Ok(res) => {
            let data = res.bytes()?;

            Ok(data.to_vec())
        }
    }
}
