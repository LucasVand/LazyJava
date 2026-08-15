use predicates::prelude::predicate;
use predicates::prelude::PredicateBooleanExt;

use crate::support::{lazy_java, Project};

#[test]
fn build_excludes_configured_java_files() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("exclude-files")?;
    let dest = &p.dir;

    // Build succeeds even though src/Broken.java does not compile, because it is excluded
    lazy_java(dest)?.args(["build"]).assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "Main.class should exist after build"
    );
    assert!(
        !dest.join("target").join("bin").join("Broken.class").exists(),
        "Broken.class should not exist because src/Broken.java is excluded"
    );

    // The excluded file should not be listed by find
    lazy_java(dest)?
        .args(["find"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Main"))
        .stdout(predicate::str::contains("Broken").not());

    // Run the included main class
    lazy_java(dest)?
        .args(["run", "Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from excluded build!"));

    Ok(())
}

#[test]
fn build_fails_without_exclude_configured() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("exclude-files")?;
    let dest = &p.dir;

    // Remove the exclude list so the broken file participates in the build
    std::fs::write(
        dest.join("lazy-java.toml"),
        "[project]\nname = \"exclude-files\"\n",
    )?;

    lazy_java(dest)?.args(["build"]).assert().failure();

    Ok(())
}