use crate::support::{lazy_java, Project};

#[test]
fn release_version_flows_to_settings_classpath_and_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("jdk-version")?;
    let dest = &p.dir;

    lazy_java(dest)?.args(["sync"]).assert().success();

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

    lazy_java(dest)?.args(["build"]).assert().success();

    let metadata = std::fs::read_to_string(dest.join("target/.lazy-java-build"))?;
    assert!(
        metadata.contains("java_version = \"11\""),
        "metadata should record the config release: {metadata}"
    );

    Ok(())
}

#[test]
fn cli_release_overrides_config_in_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("jdk-version")?;
    let dest = &p.dir;

    lazy_java(dest)?
        .args(["build", "--release", "17"])
        .assert()
        .success();

    let metadata = std::fs::read_to_string(dest.join("target/.lazy-java-build"))?;
    assert!(
        metadata.contains("java_version = \"17\""),
        "CLI --release should override the config release: {metadata}"
    );

    Ok(())
}