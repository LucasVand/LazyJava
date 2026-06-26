use crate::{
    create_maven_url,
    maven_central::{MavenError, MavenId, pom::MavenPom},
};
use reqwest::{Client, Response};

pub async fn fetch_jar(client: Client, id: &MavenId<'_>) -> Result<Vec<u8>, MavenError> {
    log::info!("Fetching JAR for {}", id);
    let res = get_from_maven(client, id, "jar").await?;

    let bytes = res.bytes().await?.to_vec();
    log::debug!("JAR downloaded: {} bytes", bytes.len());
    Ok(bytes)
}

pub async fn fetch_pom(client: Client, id: &MavenId<'_>) -> Result<MavenPom, MavenError> {
    log::info!("Fetching POM for {}", id);
    let res = get_from_maven(client, id, "pom").await?;

    let text = res.text().await?;

    MavenPom::deserialize(id, &text)
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

pub async fn get_from_maven<'a>(
    client: Client,
    id: &MavenId<'a>,
    ext: &str,
) -> Result<Response, MavenError> {
    log::debug!("Fetching Maven artifact: {} ({})", id, ext);
    let url = full_maven_url(id, ext);

    let res = client.get(&url).send().await?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(MavenError::ErrorResponse(err))
        }
        Ok(res) => Ok(res),
    }
}
