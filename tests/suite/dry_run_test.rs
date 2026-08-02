use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::Path;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn copy_fixture(name: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(name),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();
    tmp
}

#[test]
fn dry_run_build_prints_banner_and_writes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture("exclude-files");
    let dest = tmp.path().join("project");

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["--dry-run", "build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No persistent changes"))
        .stdout(predicate::str::contains("Compiling java sources"));

    assert!(
        !dest.join("target").exists(),
        "dry-run build should not create the target directory"
    );

    Ok(())
}

#[test]
fn dry_run_clean_preserves_existing_build() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture("exclude-files");
    let dest = tmp.path().join("project");

    let mut build = Command::cargo_bin("lazy-java")?;
    build.current_dir(&dest);
    build.args(["build"]);
    build.assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "build should have produced Main.class before dry-run clean"
    );

    let mut clean = Command::cargo_bin("lazy-java")?;
    clean.current_dir(&dest);
    clean.args(["--dry-run", "clean"]);
    clean.assert().success().stdout(predicate::str::contains(
        "dry-run: would remove directory",
    ));

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "dry-run clean should not remove the build output"
    );

    Ok(())
}

#[test]
fn dry_run_generate_does_not_write_pom() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture("generate-pom");
    let dest = tmp.path().join("project");

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["--dry-run", "generate", "pom"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated pom.xml"))
        .stdout(predicate::str::contains("dry-run: would write"));

    assert!(
        !dest.join("pom.xml").exists(),
        "dry-run generate should not write pom.xml"
    );

    Ok(())
}
