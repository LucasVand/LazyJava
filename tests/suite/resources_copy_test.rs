use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("resource-files")
}

#[test]
fn build_copies_resources_to_bin_and_run_reads_them() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build copies the non-java resources into bin
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    assert!(
        dest.join("target").join("bin").join("hello.txt").exists(),
        "hello.txt should be copied to bin as a resource"
    );
    assert!(
        !dest.join("target").join("bin").join("secret.txt").exists(),
        "secret.txt should be excluded from bin resources"
    );

    // Run reads the resource from the classpath
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Resource content: Hello from resource file!",
        ));

    Ok(())
}
