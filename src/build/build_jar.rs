use colored::Colorize;
use std::{
    fs, io,
    path::{Path, absolute},
    process::{Command, ExitStatus, Stdio},
};

use log::debug;

use crate::{Context, args::JarArgs, build::BuildError};

pub fn build_jar(args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    // ISSUE: cheap dry run this is not good and should be improved
    if ctx.dry_run {
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

    return Err(BuildError::NoMainClass);
}

fn build_plain_jar(output: &Path, args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    let bin = absolute(&ctx.bin)?;
    let bin_p = format!("-C {} .", bin.to_str().unwrap());
    let class_files = [bin_p.as_str()];

    let entry = entry_point(args, ctx)?;

    let manifest_str = build_manifest(&entry, ctx)?;
    let manifest_path = ctx.target.join(".build-manifest.tmp");
    fs::write(&manifest_path, &manifest_str)?;
    let result = run_jar_command(output, &manifest_path, &class_files);
    let _ = fs::remove_file(&manifest_path);
    result?;
    Ok(())
}

fn build_fat_jar(output: &Path, args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
    let temp = ctx.target.join(".fat-jar-tmp");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp)?;

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
        for entry in fs::read_dir(lib_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "jar") {
                extract_jar(&path, temp)?;
            }
        }
    }

    copy_bin(&ctx.bin, temp)?;
    merge_services(temp)?;
    let _ = fs::remove_file(temp.join("META-INF").join("MANIFEST.MF"));

    let entry = entry_point(args, ctx)?;

    let manifest_str = format!("Manifest-Version: 1.0\nMain-Class: {}\n\n", entry);

    let manifest_path = ctx.target.join(".build-manifest.tmp");
    fs::write(&manifest_path, &manifest_str)?;

    let entry = format!("-C {} .", temp.to_string_lossy());
    let result = run_jar_command(output, &manifest_path, &[entry.as_str()]);
    let _ = fs::remove_file(&manifest_path);
    result?;
    Ok(())
}

fn extract_jar(jar: &Path, dest: &Path) -> Result<(), BuildError> {
    let jar = absolute(jar)?;
    let cmd = format!(
        r#"cd "{}" && jar xf "{}""#,
        dest.to_string_lossy(),
        jar.to_string_lossy()
    );
    let status = sh(&cmd)?;
    if !status.success() {
        return Err(BuildError::CompilationErrors);
    }
    Ok(())
}

fn copy_bin(src: &Path, dest: &Path) -> Result<(), BuildError> {
    let src = absolute(src)?;
    let cmd = format!(
        r#"cp -r "{}"/* "{}""#,
        src.to_string_lossy(),
        dest.to_string_lossy()
    );
    let status = sh(&cmd)?;
    if !status.success() {
        return Err(BuildError::CompilationErrors);
    }
    Ok(())
}

pub(crate) fn merge_services(dir: &Path) -> Result<(), BuildError> {
    let services_dir = dir.join("META-INF").join("services");
    if !services_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&services_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let content = fs::read_to_string(entry.path())?;
            let mut lines: Vec<&str> = content.lines().collect();
            lines.sort();
            lines.dedup();
            fs::write(entry.path(), lines.join("\n"))?;
        }
    }
    Ok(())
}

pub(crate) fn build_manifest(entry_point: &str, ctx: &Context) -> Result<String, BuildError> {
    let mut class_path = Vec::new();

    for entry in fs::read_dir(&ctx.lib)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "jar") {
            if let Ok(relative) = path.strip_prefix(&ctx.target) {
                class_path.push(relative.to_string_lossy().to_string());
            }
        }
    }
    for entry in fs::read_dir(&ctx.lib_annotations)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "jar") {
            if let Ok(relative) = path.strip_prefix(&ctx.target) {
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

fn run_jar_command(
    output: &Path,
    manifest: &Path,
    entries: &[&str],
) -> Result<ExitStatus, BuildError> {
    let entries_str = entries.join(" ");
    let command = format!(
        r#"jar -cfm "{}" "{}" {}"#,
        output.to_string_lossy(),
        manifest.to_string_lossy(),
        entries_str
    );
    debug!("Jar Command {}", command);
    let status = sh(&command)?;
    Ok(status)
}

fn sh(command: &str) -> Result<ExitStatus, io::Error> {
    let status = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    }?;
    Ok(status)
}
