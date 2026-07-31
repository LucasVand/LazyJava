use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::Path;
use std::process::Command as StdCommand;

fn fixture_build_and_run() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("build-and-run")
}

fn fixture_with_dependency() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("with-dependency")
}

#[test]
fn build_jar_creates_executable_jar() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_build_and_run(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build first
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    // Create jar
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "jar", "--entry-point", "Main"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("build.jar").exists(),
        "build.jar should exist"
    );

    // Run the jar with plain java
    let output = StdCommand::new("java")
        .arg("-jar")
        .arg(dest.join("target").join("build.jar"))
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello world!"));
    assert!(stdout.contains("Welcome to your LazyJava project"));

    Ok(())
}

#[test]
fn build_fat_jar_self_contained() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_with_dependency(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Add a dependency and build
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    // Create fat jar
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "jar", "--entry-point", "Main", "--fat"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("build.jar").exists(),
        "build.jar should exist"
    );

    // Remove lib to prove fat jar is self-contained
    std::fs::remove_dir_all(dest.join("target").join("lib"))?;
    std::fs::remove_dir_all(dest.join("target").join("lib-annotations"))?;

    // Run the fat jar — should work without any external libs
    let output = StdCommand::new("java")
        .arg("-jar")
        .arg(dest.join("target").join("build.jar"))
        .output()?;

    assert!(
        output.status.success(),
        "fat jar should run without external lib dir, stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello world from lazy-java"));

    Ok(())
}

#[test]
fn run_jar_using_lazy_java() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_build_and_run(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build and create jar
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "jar", "--entry-point", "Main"]);
    cmd.assert().success();

    // Run via lazy-java --jar
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--jar", "--no-build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello world!"));

    Ok(())
}

#[test]
fn run_jar_errors_when_not_built() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_build_and_run(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Try running jar without building it first
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--jar", "--no-build"]);
    cmd.assert().failure();

    Ok(())
}
