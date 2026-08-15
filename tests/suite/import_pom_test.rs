use crate::support::{lazy_java, sanitize_toml, Project};

#[test]
fn import_pom_creates_lazy_java_toml() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("import-pom")?;
    let dest = &p.dir;

    lazy_java(dest)?.args(["import", "pom"]).assert().success();

    let toml_path = dest.join("lazy-java.toml");
    assert!(toml_path.exists(), "lazy-java.toml should exist");

    let toml_content = std::fs::read_to_string(toml_path)?;
    let normalized = sanitize_toml(&toml_content, dest);

    // A system-scoped dependency must have its `${project.basedir}` resolved to
    // the project root and be emitted as an absolute local dependency path.
    assert!(
        normalized.contains(r#"<ROOT>/libs/local-lib.jar"#),
        "system dependency path should resolve to <ROOT>/libs/local-lib.jar, got:\n{normalized}"
    );

    insta::assert_snapshot!("import_pom_output", normalized);

    Ok(())
}