use std::{
    io::{Cursor, Read},
};

use colored::Colorize;
use reqwest::{Client, Response, StatusCode};
use thiserror::Error;
use tokio::task::JoinSet;
use zip::{ZipArchive, result::ZipError};

use crate::{
    ContextNoConfig,
    lock_file::{LockFile, LockFileError, LockFilePackage},
    utils::{IOError, fs},
};

impl LockFile {
    pub fn fetch_packages(
        ctx: &ContextNoConfig,
        mut list: Vec<&mut LockFilePackage>,
    ) -> Result<isize, LockFileError> {
        if list.is_empty() {
            return Ok(0);
        }
        println!("    {} missing dependencies", "Fetching".bold().green());
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let client = Client::new();

        rt.block_on(async move {
            let mut changes: isize = 0;
            let mut set: JoinSet<Result<(String, Vec<u8>), LockFileError>> = JoinSet::new();

            for package in list.iter() {
                let client_clone = client.clone();
                let file_name = package.file_name.clone();
                let url = package.url.clone();
                let id = package.id.clone();
                set.spawn(async move {
                    let bin = fetch_bin(client_clone, url).await;

                    match bin {
                        Ok(bin) => {
                            println!("        {} {}", "Downloading".green().bold(), id);
                            Ok((file_name, bin))
                        }
                        Err(err) => match err {
                            FetchError::StatusCode(code) => {
                                println!(
                                    "        {} {} ({})",
                                    "Download Failed".red().bold(),
                                    id,
                                    code,
                                );
                                Err(LockFileError::DownloadFailed(code))
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

                let annotations = process_annotations(&bin).await?;
                let has_annotations = !annotations.is_empty();
                log::debug!("Annotations for {}, {:#?}", package_file_name, &annotations);
                if has_annotations {
                    let index = list
                        .iter()
                        .position(|p| p.file_name == package_file_name)
                        .expect("This must exist");

                    list[index].annotations = annotations;
                }
                changes += 1;
                let path = if has_annotations {
                    &ctx.lib_annotations
                } else {
                    &ctx.lib
                };
                let p = path.join(&package_file_name);
                fs::write(&p, bin).map_err(|s| IOError::new("writing package jar", p, s))?;
            }

            Ok(changes)
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
async fn process_annotations(bin: &Vec<u8>) -> Result<Vec<String>, ZipError> {
    let cursor = Cursor::new(bin);
    let mut archive = ZipArchive::new(cursor)?;

    let file = archive.by_name("META-INF/services/javax.annotation.processing.Processor");
    if let Err(ZipError::FileNotFound) = file {
        return Ok(Vec::new());
    }
    let mut file = file?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let processors = contents
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    Ok(processors)
}
