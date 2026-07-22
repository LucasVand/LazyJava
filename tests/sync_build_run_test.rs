use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("with-dependency")
}

#[test]
fn sync_build_run_with_maven_dependency() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Step 1: Add dependency — resolves from Maven, downloads JAR, creates lock file
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3"]);
    cmd.assert().success();

    assert!(
        dest.join("lazy-java.lock").exists(),
        "lock file should exist after add"
    );

    assert!(
        dest.join("target").join("lib").is_dir(),
        "lib dir should exist after add"
    );

    // Step 2: Build
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "Main.class should exist after build"
    );

    // Step 3: Run and assert output from commons-lang3 usage
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello world from lazy-java"))
        .stdout(predicate::str::contains("hello wor..."))
        .stdout(predicate::str::contains("avaj-yzal morf dlrow olleh"));

    Ok(())
}
