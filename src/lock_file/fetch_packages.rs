use std::{fs, path::Path};

use colored::Colorize;
use reqwest::{Client, Response};
use tokio::task::JoinSet;

use crate::lock_file::{LockFile, LockFileError, LockFilePackage};

impl LockFile {
    pub fn fetch_packages(lib: &Path, list: Vec<&LockFilePackage>) -> Result<isize, LockFileError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let client = Client::new();

        rt.block_on(async move {
            let mut changes: isize = 0;
            let mut set: JoinSet<Result<(String, Vec<u8>), LockFileError>> = JoinSet::new();

            for package in list.into_iter() {
                let client_clone = client.clone();
                let file_name = package.file_name.clone();
                let url = package.url.clone();
                let id = package.id.clone();
                set.spawn(async move {
                    println!("    {} {}", "Downloading".green().bold(), id);
                    let bin = fetch_bin(client_clone, url).await?;

                    return Ok((file_name, bin));
                });
            }

            while let Some(Ok(result)) = set.join_next().await {
                let (package_file_name, bin) = result?;

                changes += 1;
                fs::write(lib.join(package_file_name), bin)?;
            }
            return Ok(changes);
        })
    }
}
async fn fetch_bin(client: Client, url: String) -> Result<Vec<u8>, LockFileError> {
    let res: Response = client.get(url).send().await?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(LockFileError::RequestError(err))
        }
        Ok(res) => {
            let bytes = res.bytes().await?;
            Ok(bytes.to_vec())
        }
    }
}
