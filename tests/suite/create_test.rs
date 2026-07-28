use assert_cmd::Command;
use std::path::Path;

fn project_dir(tmp: &Path, name: &str) -> std::path::PathBuf {
    tmp.join(name)
}

#[test]
fn create_project_creates_all_expected_files() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let name = "test-project";

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(tmp.path());
    cmd.args(["create", "--name", name, "--git", "false"]);
    cmd.assert().success();

    let dir = project_dir(tmp.path(), name);
    assert!(dir.exists(), "project directory should exist");
    assert!(
        dir.join("lazy-java.toml").exists(),
        "lazy-java.toml should exist"
    );
    assert!(dir.join("pom.xml").exists(), "pom.xml should exist");
    assert!(
        dir.join("src").join("Main.java").exists(),
        "Main.java should exist"
    );
    assert!(dir.join("target").exists(), "target dir should exist");
    assert!(
        dir.join("target").join("bin").exists(),
        "target/bin should exist"
    );
    assert!(
        dir.join("target").join("lib").exists(),
        "target/lib should exist"
    );
    assert!(
        !dir.join(".gitignore").exists(),
        ".gitignore should not exist with --git false"
    );

    Ok(())
}

#[test]
fn create_project_bare_skips_main_java() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let name = "bare-project";

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(tmp.path());
    cmd.args(["create", "--name", name, "--git", "false", "--bare"]);
    cmd.assert().success();

    let dir = project_dir(tmp.path(), name);
    assert!(dir.exists(), "project directory should exist");
    assert!(
        !dir.join("src").join("Main.java").exists(),
        "Main.java should NOT exist with --bare"
    );
    assert!(dir.join("src").exists(), "src dir should still exist");

    Ok(())
}
