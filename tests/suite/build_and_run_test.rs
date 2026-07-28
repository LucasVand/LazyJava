use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("build-and-run")
}

#[test]
fn build_and_run_produces_expected_output() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    // Copy fixture to temp dir
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "Main.class should exist after build"
    );

    // Run
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello world!"))
        .stdout(predicate::str::contains("Welcome to your LazyJava project"));

    Ok(())
}
