use assert_cmd::Command;
use predicates::prelude::predicate;
use predicates::prelude::PredicateBooleanExt;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("exclude-files")
}

#[test]
fn build_excludes_configured_java_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build succeeds even though src/Broken.java does not compile, because it is excluded
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "Main.class should exist after build"
    );
    assert!(
        !dest.join("target").join("bin").join("Broken.class").exists(),
        "Broken.class should not exist because src/Broken.java is excluded"
    );

    // The excluded file should not be listed by find
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["find"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Main"))
        .stdout(predicate::str::contains("Broken").not());

    // Run the included main class
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello from excluded build!"));

    Ok(())
}

#[test]
fn build_fails_without_exclude_configured() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Remove the exclude list so the broken file participates in the build
    std::fs::write(
        dest.join("lazy-java.toml"),
        "[project]\nname = \"exclude-files\"\n",
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().failure();

    Ok(())
}
