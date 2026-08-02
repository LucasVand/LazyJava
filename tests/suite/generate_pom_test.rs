use assert_cmd::Command;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("generate-pom")
}

#[test]
fn generate_pom_includes_deps_and_annotation_processors() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "com.google.auto.value", "auto-value-annotations"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "com.google.auto.value", "auto-value"]);
    cmd.assert().success();

    assert!(
        dest.join("lazy-java.lock").exists(),
        "lock file should exist after add"
    );

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["generate", "pom"]);
    cmd.assert().success();

    let pom_path = dest.join("pom.xml");
    assert!(pom_path.exists(), "pom.xml should exist");

    let pom_content = std::fs::read_to_string(pom_path)?;

    insta::assert_snapshot!("generate_pom_output", pom_content);

    Ok(())
}
