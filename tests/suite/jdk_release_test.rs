use assert_cmd::Command;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("jdk-version")
}

fn copy_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();
    tmp
}

#[test]
fn release_version_flows_to_settings_classpath_and_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = copy_fixture();
    let dest = tmp.path().join("project");

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["sync"]);
    cmd.assert().success();

    let settings = std::fs::read_to_string(dest.join(".settings/org.eclipse.core.prefs"))?;
    assert!(
        settings.contains("org.eclipse.jdt.core.compiler.source=11"),
        "settings should pin source to the configured release: {settings}"
    );
    assert!(
        settings.contains("org.eclipse.jdt.core.compiler.compliance=11"),
        "settings should pin compliance to the configured release: {settings}"
    );
    assert!(
        settings.contains("org.eclipse.jdt.core.compiler.codegen.targetPlatform=11"),
        "settings should pin targetPlatform to the configured release: {settings}"
    );

    let classpath = std::fs::read_to_string(dest.join(".classpath"))?;
    assert!(
        classpath.contains("JavaSE-11"),
        "classpath should reference the configured JRE version: {classpath}"
    );

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    let metadata = std::fs::read_to_string(dest.join("target/.lazy-java-build"))?;
    assert!(
        metadata.contains("java_version = \"11\""),
        "metadata should record the config release: {metadata}"
    );

    Ok(())
}

#[test]
fn cli_release_overrides_config_in_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture();
    let dest = tmp.path().join("project");

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "--release", "17"]);
    cmd.assert().success();

    let metadata = std::fs::read_to_string(dest.join("target/.lazy-java-build"))?;
    assert!(
        metadata.contains("java_version = \"17\""),
        "CLI --release should override the config release: {metadata}"
    );

    Ok(())
}
