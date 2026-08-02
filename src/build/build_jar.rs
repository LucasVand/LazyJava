use colored::Colorize;
use std::{
    io,
    path::{Path, absolute},
    process::{Command, Stdio},
};
use walkdir::WalkDir;

use log::debug;

use crate::{Context, args::JarArgs, build::BuildError, utils::{IOError, fs, GlobalContext}};

pub fn build_jar(args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    // ISSUE: cheap dry run this is not good and should be improved
    if GlobalContext::is_dry_run() {
        let entry = entry_point(args, ctx).unwrap_or_default();
        println!(
            "{} jar with entry point: {}",
            "Creating".bold().green(),
            entry
        );
        return Ok(());
    }

    let output = ctx.target.join("build.jar");

    if args.fat {
        build_fat_jar(&output, args, ctx)?;
    } else {
        build_plain_jar(&output, args, ctx)?;
    }

    println!(
        "{} {} ({})",
        "Created".bold().green(),
        if args.fat { "fat jar" } else { "jar" },
        output.display()
    );

    Ok(())
}
fn entry_point(args: &JarArgs, ctx: &Context) -> Result<String, BuildError> {
    if let Some(point) = &args.entry_point {
        return Ok(point.clone());
    }

    if let Some(setup) = ctx.config.setup()
        && let Some(point) = setup.main_class()
    {
        return Ok(point);
    }

    Err(BuildError::NoMainClass)
}

fn build_plain_jar(output: &Path, args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    let bin = absolute(&ctx.bin).map_err(|e| IOError::new("resolving bin path", &ctx.bin, e))?;
    let class_files = [bin.as_path()];

    let entry = entry_point(args, ctx)?;

    let manifest_str = build_manifest(&entry, ctx)?;
    let manifest_path = ctx.target.join(".build-manifest.tmp");

    fs::write(&manifest_path, &manifest_str)
        .map_err(|e| IOError::new("writing build manifest", &manifest_path, e))?;

    let result = run_jar_command(output, &manifest_path, &class_files);
    let _ = fs::remove_file(&manifest_path);
    result
}

fn build_fat_jar(output: &Path, args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    let temp = ctx.target.join(".fat-jar-tmp");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).map_err(|e| IOError::new("creating fat jar directory", &temp, e))?;

    let r = build_fat_jar_inner(output, args, ctx, &temp);

    let _ = fs::remove_dir_all(&temp);
    r
}

fn build_fat_jar_inner(
    output: &Path,
    args: &JarArgs,
    ctx: &Context,
    temp: &Path,
) -> Result<(), BuildError> {
    let lib_dirs = [&ctx.lib, &ctx.lib_annotations];
    for lib_dir in &lib_dirs {
        for entry in WalkDir::new(lib_dir) {
            let entry =
                entry.map_err(|e| IOError::new("reading library entry", lib_dir, e.into()))?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "jar") {
                extract_jar(path, temp)?;
            }
        }
    }

    copy_bin(&ctx.bin, temp)?;
    merge_services(temp)?;
    let _ = fs::remove_file(temp.join("META-INF").join("MANIFEST.MF"));

    let entry = entry_point(args, ctx)?;

    let manifest_str = format!("Manifest-Version: 1.0\nMain-Class: {}\n\n", entry);

    let manifest_path = ctx.target.join(".build-manifest.tmp");
    fs::write(&manifest_path, &manifest_str)
        .map_err(|e| IOError::new("writing build manifest", &manifest_path, e))?;

    let result = run_jar_command(output, &manifest_path, &[temp]);
    let _ = fs::remove_file(&manifest_path);
    result
}

fn extract_jar(jar: &Path, dest: &Path) -> Result<(), BuildError> {
    let jar = absolute(jar).map_err(|e| IOError::new("resolving jar path", jar, e))?;

    let status = Command::new("jar")
        .current_dir(dest)
        .arg("xf")
        .arg(&jar)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| IOError::new("extracting jar", &jar, e))?;
    if !status.success() {
        return Err(BuildError::JarCreationError);
    }
    Ok(())
}

fn copy_bin(src: &Path, dest: &Path) -> Result<(), BuildError> {
    let src = absolute(src).map_err(|e| IOError::new("resolving bin path", src, e))?;
    copy_dir_recursively(&src, dest).map_err(|e| IOError::new("copying bin directory", dest, e))?;
    Ok(())
}

fn copy_dir_recursively(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursively(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

pub(crate) fn merge_services(dir: &Path) -> Result<(), BuildError> {
    let services_dir = dir.join("META-INF").join("services");
    if !services_dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(&services_dir) {
        let entry = entry.map_err(|e| {
            BuildError::IoError(IOError::new(
                "reading services entry",
                &services_dir,
                e.into(),
            ))
        })?;
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path())
                .map_err(|e| IOError::new("reading services file", entry.path(), e))?;
            let mut lines: Vec<&str> = content.lines().collect();
            lines.sort();
            lines.dedup();
            fs::write(entry.path(), lines.join("\n"))
                .map_err(|e| IOError::new("writing services file", entry.path(), e))?;
        }
    }
    Ok(())
}

pub(crate) fn build_manifest(entry_point: &str, ctx: &Context) -> Result<String, BuildError> {
    let mut class_path = Vec::new();

    for lib_dir in [&ctx.lib, &ctx.lib_annotations] {
        for entry in WalkDir::new(lib_dir) {
            let entry = entry.map_err(|e| {
                BuildError::IoError(IOError::new("reading library entry", lib_dir, e.into()))
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "jar")
                && let Ok(relative) = path.strip_prefix(&ctx.target) {
                    class_path.push(relative.to_string_lossy().to_string());
                }
        }
    }

    let mut manifest = format!("Manifest-Version: 1.0\nMain-Class: {}\n", entry_point);

    if !class_path.is_empty() {
        manifest.push_str("Class-Path: ");
        let mut line_len = 12;
        for entry in &class_path {
            let entry_len = entry.len() + 1;
            if line_len + entry_len > 72 {
                manifest.push('\n');
                manifest.push(' ');
                line_len = 1 + entry.len();
            } else {
                line_len += entry_len;
            }
            manifest.push_str(entry);
            manifest.push(' ');
        }
        manifest.push('\n');
    }

    manifest.push('\n');
    Ok(manifest)
}

fn run_jar_command(output: &Path, manifest: &Path, dirs: &[&Path]) -> Result<(), BuildError> {
    let mut cmd = Command::new("jar");
    cmd.arg("-cfm").arg(output).arg(manifest);
    for dir in dirs {
        cmd.arg("-C").arg(dir).arg(".");
    }
    debug!("Jar Command {:?}", cmd);
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| IOError::new("running jar command", output, e))?;
    if !status.success() {
        return Err(BuildError::JarCreationError);
    }
    Ok(())
}
