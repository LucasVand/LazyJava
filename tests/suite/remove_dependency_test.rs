use assert_cmd::Command;
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("with-dependency")
}

/// Replaces the absolute project root with `<ROOT>` so lock snapshots are
/// deterministic across machines. The lock file embeds each jar's `path`,
/// which on the generating machine is an absolute path under the temp project
/// dir, in either canonical (`/private/var/...`) or symlinked (`/var/...`)
/// form, so both are substituted.
fn sanitize_lock(content: &str, root: &Path) -> String {
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
fn remove_dependency_cleans_config_and_lock() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");

    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Add commons-lang3
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args([
        "add",
        "org.springframework.boot",
        "spring-boot-starter-jdbc",
    ]);
    cmd.assert().success();

    let toml_after_add = std::fs::read_to_string(dest.join("lazy-java.toml"))?;
    insta::assert_snapshot!("add_dependency_config", toml_after_add);

    let lock_after_add = sanitize_lock(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("add_dependency_lock", lock_after_add);

    // Remove commons-lang3
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args([
        "remove",
        "org.springframework.boot",
        "spring-boot-starter-jdbc",
    ]);
    cmd.assert().success();

    let toml_after_remove = std::fs::read_to_string(dest.join("lazy-java.toml"))?;
    insta::assert_snapshot!("remove_dependency_config", toml_after_remove);

    let lock_after_remove = sanitize_lock(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("remove_dependency_lock", lock_after_remove);

    Ok(())
}
