use assert_cmd::Command;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("import-pom")
}

/// Replace the absolute temp project root with `<ROOT>` so the imported toml
/// snapshot is deterministic across machines (canonical and symlinked forms).
fn sanitize_toml(content: &str, root: &Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    let canonical = std::fs::canonicalize(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| root.clone());

    let mut normalized = content.replace('\\', "/");
    if !canonical.is_empty() && canonical != root {
        normalized = normalized.replace(&canonical, "<ROOT>");
    }
    normalized.replace(&root, "<ROOT>")
}

#[test]
fn import_pom_creates_lazy_java_toml() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["import", "pom"]);
    cmd.assert().success();

    let toml_path = dest.join("lazy-java.toml");
    assert!(toml_path.exists(), "lazy-java.toml should exist");

    let toml_content = std::fs::read_to_string(toml_path)?;
    let normalized = sanitize_toml(&toml_content, &dest);

    // A system-scoped dependency must have its `${project.basedir}` resolved to
    // the project root and be emitted as an absolute local dependency path.
    assert!(
        normalized.contains(r#"path = "<ROOT>/libs/local-lib.jar""#),
        "system dependency path should resolve to <ROOT>/libs/local-lib.jar, got:\n{normalized}"
    );

    insta::assert_snapshot!("import_pom_output", normalized);

    Ok(())
}
