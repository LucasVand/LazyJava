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

    // Place a file OUTSIDE the project root and reference it with a relative
    // path that escapes the root (`../outside.txt`), to exercise true external
    // file support (paths that point outside of the project root).
    let outside = tmp.path().join("outside.txt");
    std::fs::write(&outside, "hello from outside the project root\n")?;

    let config_path = dest.join("lazy-java.toml");
    let mut config = std::fs::read_to_string(&config_path)?;
    config = config.replace(
        r#"external = ["external.txt"]"#,
        r#"external = ["external.txt", "../outside.txt"]"#,
    );
    std::fs::write(&config_path, config)?;

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
    assert!(
        dest.join("target")
            .join("bin")
            .join("external.txt")
            .exists(),
        "external.txt should be copied to bin as an external resource"
    );
    assert!(
        dest.join("target").join("bin").join("outside.txt").exists(),
        "a file referenced from outside the project root should be copied to bin and kept"
    );

    // Run reads the resources from the classpath
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Resource content: Hello from resource file!",
        ))
        .stdout(predicate::str::contains(
            "External: External resource content",
        ));

    Ok(())
}
