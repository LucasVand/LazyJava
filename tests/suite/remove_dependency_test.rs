use assert_cmd::Command;
use std::path::Path;

use crate::support::sanitize_toml;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("with-dependency")
}

#[test]
fn remove_dependency_cleans_config_and_lock() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Add commons-lang3
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args([
        "add",
        "org.springframework.boot",
        "spring-boot-starter-jdbc",
    ]);
    cmd.assert().success();

    let toml_after_add = std::fs::read_to_string(dest.join("lazy-java.toml"))?;
    insta::assert_snapshot!("add_dependency_config", toml_after_add);

    let lock_after_add = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("add_dependency_lock", lock_after_add);

    // Remove commons-lang3
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args([
        "remove",
        "org.springframework.boot",
        "spring-boot-starter-jdbc",
    ]);
    cmd.assert().success();

    let toml_after_remove = std::fs::read_to_string(dest.join("lazy-java.toml"))?;
    insta::assert_snapshot!("remove_dependency_config", toml_after_remove);

    let lock_after_remove = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("remove_dependency_lock", lock_after_remove);

    Ok(())
}
