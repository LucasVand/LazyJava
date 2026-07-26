use assert_cmd::Command;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("import-pom")
}

#[test]
fn import_pom_creates_lazy_java_toml() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["import", "pom"]);
    cmd.assert().success();

    let toml_path = dest.join("lazy-java.toml");
    assert!(toml_path.exists(), "lazy-java.toml should exist");

    let toml_content = std::fs::read_to_string(toml_path)?;

    insta::assert_snapshot!("import_pom_output", toml_content);

    Ok(())
}
