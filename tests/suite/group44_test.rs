use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("group44")
}

fn copy_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();
    tmp
}

fn run(dest: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(dest);
    cmd.args(args);
    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "`lazy-java {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

/// Normalizes command output for deterministic snapshot comparison:
/// - absolute paths under the temp project dir become `<ROOT>`
/// - the non-deterministic `Compiled in X.XXs` line becomes `Compiled in <TIME>s`
/// - lines are sorted, since graph and HashSet iteration order is not stable
fn normalize_output(output: &str, root: &Path) -> String {
    // Normalize separators up front so the project root matches on both
    // Unix (`/`) and Windows (`\`) before substituting `<ROOT>`.
    let root_path = root.to_string_lossy().replace('\\', "/");
    let canonical_root = std::fs::canonicalize(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| root_path.clone());

    let mut lines: Vec<String> = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let l = if l.starts_with("Compiled in") {
                "Compiled in <TIME>s"
            } else {
                l
            };
            let l = l.replace('\\', "/");
            // replace the canonical (symlink-resolved) path first so a
            // `/private/var/...` path does not leave a `/private` prefix
            l.replace(&canonical_root, "<ROOT>")
                .replace(&root_path, "<ROOT>")
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

#[test]
fn group44_full_build_graph_and_incrimental_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture();
    let dest = tmp.path().join("project");

    // Baseline: full build
    insta::assert_snapshot!(
        "full_build_output",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // Dependency and dependant graphs
    insta::assert_snapshot!(
        "dependancy_graph",
        normalize_output(&run(&dest, &["build", "dependancies"])?, &dest)
    );
    insta::assert_snapshot!(
        "dependants_graph",
        normalize_output(&run(&dest, &["build", "dependants"])?, &dest)
    );

    // Nothing should be stale right after a build
    insta::assert_snapshot!(
        "stale_after_full_build",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // Edit a low-level utility that is imported across the whole UI,
    // expecting the incremental build to fan out to every transitive
    // dependant.
    let color_manager = dest.join("src/utils/ColorManager.java");
    let mut contents = std::fs::read_to_string(&color_manager)?;
    contents.push_str("\n    // modified by the group44 e2e test\n");
    std::fs::write(&color_manager, contents)?;

    insta::assert_snapshot!(
        "stale_after_color_manager_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // The incremental build must recompile the edited file plus all its
    // transitive dependants, and still succeed.
    insta::assert_snapshot!(
        "incrimental_build_after_color_manager_edit",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // After the incremental build nothing should be stale.
    insta::assert_snapshot!(
        "stale_after_incrimental_build",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // A repeated build with no further changes skips compilation entirely.
    insta::assert_snapshot!(
        "incrimental_build_no_changes",
        normalize_output(&run(&dest, &["build"])?, &dest)
    );

    Ok(())
}