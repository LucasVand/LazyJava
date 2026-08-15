use predicates::prelude::predicate;

use crate::support::{lazy_java, Project};

#[test]
fn build_and_run_produces_expected_output() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("build-and-run")?;
    let dest = &p.dir;

    // Build
    lazy_java(dest)?.args(["build"]).assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "Main.class should exist after build"
    );

    // Run
    lazy_java(dest)?
        .args(["run", "Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello world!"))
        .stdout(predicate::str::contains("Welcome to your LazyJava project"));

    Ok(())
}