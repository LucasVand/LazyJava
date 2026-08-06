#![allow(clippy::disallowed_methods)]

use std::{
    fs as std_fs,
    io,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::utils::GlobalContext;

pub use std_fs::{DirEntry, Metadata, ReadDir};

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    if GlobalContext::is_dry_run() {
        println!("{}: would write {}", "dry-run".red().bold(), path.display());
        return Ok(());
    }
    std_fs::write(path, contents)
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    if GlobalContext::is_dry_run() {
        println!(
            "{}: would create directory {}",
            "dry-run".red().bold(),
            path.display()
        );
        return Ok(());
    }
    std_fs::create_dir(path)
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    if GlobalContext::is_dry_run() {
        println!(
            "{}: would create directory {}",
            "dry-run".red().bold(),
            path.display()
        );
        return Ok(());
    }
    std_fs::create_dir_all(path)
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    if GlobalContext::is_dry_run() {
        println!("{}: would remove file {}", "dry-run".red().bold(), path.display());
        return Ok(());
    }
    std_fs::remove_file(path)
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    if GlobalContext::is_dry_run() {
        println!(
            "{}: would remove directory {}",
            "dry-run".red().bold(),
            path.display()
        );
        return Ok(());
    }
    std_fs::remove_dir_all(path)
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();
    if GlobalContext::is_dry_run() {
        println!(
            "{}: would copy {} to {}",
            "dry-run".red().bold(),
            from.display(),
            to.display()
        );
        return Ok(0);
    }
    std_fs::copy(from, to)
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    std_fs::read_to_string(path)
}

pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    std_fs::read(path)
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<std_fs::ReadDir> {
    std_fs::read_dir(path)
}

pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<std_fs::Metadata> {
    std_fs::metadata(path)
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    std_fs::canonicalize(path)
}

pub fn exists<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    std_fs::exists(path)
}
