use std::{fs, path::Path};

use colored::Colorize;
use reqwest::{Client, Response, StatusCode};
use thiserror::Error;
use tokio::task::JoinSet;

use crate::lock_file::{LockFile, LockFileError, LockFilePackage};

impl LockFile {
    pub fn fetch_packages(
        lib: &Path,
        list: Vec<&LockFilePackage>,
        dry_run: bool,
    ) -> Result<isize, LockFileError> {
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
                    let bin = fetch_bin(client_clone, url).await;

                    match bin {
                        Ok(bin) => {
                            println!("    {} {}", "Downloading".green().bold(), id);
                            return Ok((file_name, bin));
                        }
                        Err(err) => match err {
                            FetchError::StatusCode(code) => {
                                println!(
                                    "    {} {} ({})",
                                    "Download Failed".red().bold(),
                                    id,
                                    code,
                                );
                                Err(LockFileError::NoFound)
                            }
                            FetchError::ReqwestError(err) => Err(LockFileError::RequestError(err)),
                        },
                    }
                });
            }
            let results = set.join_all().await;

            let mut result_errors = Vec::new();
            let mut clean_results = Vec::new();
            results.into_iter().for_each(|result| match result {
                Ok(res) => clean_results.push(res),
                Err(err) => result_errors.push(err),
            });

            if !result_errors.is_empty() {
                return Err(LockFileError::FetchError(result_errors));
            }

            for result in clean_results {
                let (package_file_name, bin) = result;

                changes += 1;
                if !dry_run {
                    fs::write(lib.join(package_file_name), bin)?;
                }
            }

            return Ok(changes);
        })
    }
}

#[derive(Debug, Error)]
enum FetchError {
    #[error("Server returned with a bad status code")]
    StatusCode(StatusCode),
    #[error("Could not make the request")]
    ReqwestError(#[from] reqwest::Error),
}

async fn fetch_bin(client: Client, url: String) -> Result<Vec<u8>, FetchError> {
    let res: Response = client.get(url).send().await?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(FetchError::StatusCode(err.status().unwrap()))
        }
        Ok(res) => {
            let bytes = res.bytes().await?;
            Ok(bytes.to_vec())
        }
    }
}
