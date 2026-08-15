use predicates::prelude::predicate;

use crate::support::{lazy_java, Project};

#[test]
fn dry_run_build_prints_banner_and_writes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("exclude-files")?;
    let dest = &p.dir;

    lazy_java(dest)?
        .args(["--dry-run", "build"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No persistent changes"))
        .stdout(predicate::str::contains("Compiling java sources"));

    assert!(
        !dest.join("target").exists(),
        "dry-run build should not create the target directory"
    );

    Ok(())
}

#[test]
fn dry_run_clean_preserves_existing_build() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("exclude-files")?;
    let dest = &p.dir;

    lazy_java(dest)?.args(["build"]).assert().success();

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "build should have produced Main.class before dry-run clean"
    );

    lazy_java(dest)?
        .args(["--dry-run", "clean"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run: would remove directory"));

    assert!(
        dest.join("target").join("bin").join("Main.class").exists(),
        "dry-run clean should not remove the build output"
    );

    Ok(())
}

#[test]
fn dry_run_generate_does_not_write_pom() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("generate-pom")?;
    let dest = &p.dir;

    lazy_java(dest)?
        .args(["--dry-run", "generate", "pom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated pom.xml"))
        .stdout(predicate::str::contains("dry-run: would write"));

    assert!(
        !dest.join("pom.xml").exists(),
        "dry-run generate should not write pom.xml"
    );

    Ok(())
}