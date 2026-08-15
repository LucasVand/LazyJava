use predicates::prelude::predicate;

use crate::support::{lazy_java, Project};

#[test]
fn build_copies_resources_to_bin_and_run_reads_them() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("resource-files")?;
    let dest = &p.dir;

    // Place a file OUTSIDE the project root and reference it with a relative
    // path that escapes the root (`../outside.txt`), to exercise true external
    // file support (paths that point outside of the project root).
    let outside = p.root().join("outside.txt");
    std::fs::write(&outside, "hello from outside the project root\n")?;

    let config_path = dest.join("lazy-java.toml");
    let mut config = std::fs::read_to_string(&config_path)?;
    config = config.replace(
        r#"external = ["external.txt"]"#,
        r#"external = ["external.txt", "../outside.txt"]"#,
    );
    std::fs::write(&config_path, config)?;

    // Build copies the non-java resources into bin
    lazy_java(dest)?.args(["build"]).assert().success();

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
    lazy_java(dest)?
        .args(["run", "Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Resource content: Hello from resource file!",
        ))
        .stdout(predicate::str::contains(
            "External: External resource content",
        ));

    Ok(())
}